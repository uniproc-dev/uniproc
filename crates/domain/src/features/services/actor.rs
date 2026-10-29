use std::rc::Rc;
use std::sync::Arc;

use app_contracts::features::agents::{
    ActionFailure, ActionOutcome, WindowsAction, WindowsReportMessage, WindowsServiceStats,
};
use app_contracts::features::services::{
    Command, Deselect, DismissFailure, Select, ServiceActionKind, ServiceColumn, ServiceRow, ServicesMsg,
    ServicesState, Sort, RunNewTask, OpenServices,
    ServiceOperation,
};
use guinea::prelude::*;
use uniproc_windows_agent::api::SERVICE_NAME;

use crate::features::agents::actions;
use crate::features::processes::shell::ShellRequest;
use crate::features::system::tools::Launch;
use app_contracts::features::window::PressedAway;

pub struct Acted {
    name: String,
    action: WindowsAction,
    target: Arc<str>,
    outcome: ActionOutcome,
}

#[derive(Debug)]
pub struct ServicesActor {
    push: Push<ServicesState>,
    rows: Rc<[ServiceRow]>,
    sort_column: ServiceColumn,
    descending: bool,
    selected: Option<String>,
    operations: Vec<ServiceOperation>,
    shell: fn(ShellRequest),
    launch: fn(Launch),
}

impl ServicesActor {
    pub fn new(push: Push<ServicesState>, shell: fn(ShellRequest), launch: fn(Launch)) -> Self {
        Self {
            push,
            rows: Rc::from(Vec::new()),
            sort_column: ServiceColumn::Name,
            descending: false,
            selected: None,
            operations: Vec::new(),
            shell,
            launch,
        }
    }

    fn publish_rows(&self) {
        self.push.send(ServicesMsg::SetRows {
            rows: self.rows.clone(),
        });
    }

    fn publish_operations(&self) {
        self.push.send(ServicesMsg::SetOperations(self.operations.clone().into()));
    }

    fn resort(&mut self) {
        let mut rows = self.rows.to_vec();
        sort_rows(&mut rows, self.sort_column, self.descending);
        self.rows = Rc::from(rows);
    }
}

actor! {
    ServicesActor {
        handlers {
            Sort,
            Select,
            Deselect,
            Command => { task Acted }
            Acted,
            DismissFailure,
            RunNewTask,
            OpenServices,
        }
        subscribes { WindowsReportMessage, PressedAway }
    }
}

#[handler]
fn on_pressed_away(In(this, _, _): In<ServicesActor, PressedAway>) {
    this.selected = None;
    this.push.send(ServicesMsg::SetSelected(None));
}

#[handler]
fn run_new_task(In(this, _, _): In<ServicesActor, RunNewTask>) {
    (this.shell)(ShellRequest::RunNewTask);
}

#[handler]
fn open_services(In(this, _, _): In<ServicesActor, OpenServices>) {
    (this.launch)(Launch { file: "services.msc".into(), args: String::new() });
}

#[handler]
fn sort(In(this, Sort(column), _): In<ServicesActor, Sort>) {
    if this.sort_column == column {
        this.descending = !this.descending;
    } else {
        this.sort_column = column;
        this.descending = false;
    }
    this.resort();
    this.push.send(ServicesMsg::SetSort {
        column: this.sort_column,
        descending: this.descending,
    });
    this.publish_rows();
}

#[handler]
fn select(In(this, Select(name), _): In<ServicesActor, Select>) {
    this.selected = Some(name.clone());
    this.push.send(ServicesMsg::SetSelected(Some(name)));
}

#[handler]
fn deselect(In(this, _msg, _): In<ServicesActor, Deselect>) {
    this.selected = None;
    this.push.send(ServicesMsg::SetSelected(None));
}

#[handler]
fn command(In(this, Command(kind), cx): In<ServicesActor, Command>) {
    let Some(name) = this.selected.clone() else {
        return;
    };
    if this.operations.iter().any(|op| op.name.as_ref() == name) {
        return;
    }
    let stops = matches!(kind, ServiceActionKind::Stop | ServiceActionKind::Pause | ServiceActionKind::Restart);
    if stops && *name == *SERVICE_NAME {
        return;
    }
    let target = this
        .rows
        .iter()
        .find(|row| *row.name == *name)
        .map_or_else(|| Arc::from(name.as_str()), |row| row.display_name.clone());
    let action = match kind {
        ServiceActionKind::Start => WindowsAction::ServiceStart { name: name.clone() },
        ServiceActionKind::Stop => WindowsAction::ServiceStop { name: name.clone() },
        ServiceActionKind::Pause => WindowsAction::ServicePause { name: name.clone() },
        ServiceActionKind::Resume => WindowsAction::ServiceResume { name: name.clone() },
        ServiceActionKind::Restart => WindowsAction::ServiceRestart { name: name.clone() },
    };
    this.operations.push(ServiceOperation { name: name.clone().into(), display_name: target.clone() });
    this.publish_operations();
    let remote = cx.async_ctx();
    cx.task().spawn(async move {
        let outcome = actions::request(remote, action.clone()).await;
        Acted { name, action, target, outcome }
    });
}

#[handler]
fn on_windows_report(In(this, msg, _): In<ServicesActor, WindowsReportMessage>) {
    let WindowsReportMessage::Report(report) = msg else {
        return;
    };
    let mut rows: Vec<ServiceRow> = report.services.iter().map(to_row).collect();
    sort_rows(&mut rows, this.sort_column, this.descending);
    this.rows = Rc::from(rows);

    if let Some(selected) = &this.selected
        && !this.rows.iter().any(|r| *r.name == **selected)
    {
        this.selected = None;
        this.push.send(ServicesMsg::SetSelected(None));
    }

    this.publish_rows();
}

#[handler]
fn on_acted(In(this, Acted { name, action, target, outcome }, _): In<ServicesActor, Acted>) {
    this.operations.retain(|op| op.name.as_ref() != name);
    this.publish_operations();
    if outcome == ActionOutcome::Done {
        return;
    }
    tracing::warn!(?action, ?outcome, "the action did not go through");
    this.push.send(ServicesMsg::SetFailure(Some(ActionFailure { action, target, outcome })));
}

#[handler]
fn dismiss_failure(In(this, _msg, _): In<ServicesActor, DismissFailure>) {
    this.push.send(ServicesMsg::SetFailure(None));
}

fn to_row(svc: &WindowsServiceStats) -> ServiceRow {
    ServiceRow {
        name: svc.name.clone(),
        display_name: svc.display_name.clone(),
        pid: svc.pid,
        state: svc.state,
        group: svc.load_group.clone(),
        description: svc.description.clone(),
        image_path: svc.image_path.clone(),
        startup_type: svc.startup_type,
        delayed_auto: svc.delayed_auto,
        account: svc.account.clone(),
        service_type: svc.service_type,
        accepted_controls: svc.accepted_controls,
    }
}

fn folded(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().flat_map(char::to_lowercase)
}

fn sort_rows(rows: &mut [ServiceRow], column: ServiceColumn, descending: bool) {
    rows.sort_by(|a, b| {
        let ord = match column {
            ServiceColumn::Status => a.state.id().cmp(b.state.id()),
            ServiceColumn::Group => a.group.cmp(&b.group),
            ServiceColumn::Pid => a.pid.cmp(&b.pid),
            ServiceColumn::Name => folded(&a.display_name).cmp(folded(&b.display_name)),
            ServiceColumn::Description => folded(&a.description).cmp(folded(&b.description)),
            ServiceColumn::ServiceName => folded(&a.name).cmp(folded(&b.name)),
            ServiceColumn::ImagePath => folded(&a.image_path).cmp(folded(&b.image_path)),
            ServiceColumn::StartupType => (a.startup_type as u8, a.delayed_auto).cmp(&(b.startup_type as u8, b.delayed_auto)),
            ServiceColumn::Account => folded(&a.account).cmp(folded(&b.account)),
            ServiceColumn::ServiceType => a.service_type.cmp(&b.service_type),
        };
        let ord = ord.then_with(|| a.name.cmp(&b.name));
        if descending { ord.reverse() } else { ord }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use app_contracts::features::agents::WindowsServiceState;

    fn row(name: &str, state: WindowsServiceState) -> ServiceRow {
        ServiceRow {
            name: name.into(),
            display_name: name.into(),
            pid: 0,
            state,
            group: "".into(),
            description: "".into(),
            image_path: "".into(),
            startup_type: Default::default(),
            delayed_auto: false,
            account: "".into(),
            service_type: 0,
            accepted_controls: 0,
        }
    }

    #[test]
    fn sorts_by_name_case_insensitively_by_default() {
        let mut rows = vec![
            row("beta", WindowsServiceState::Running),
            row("Alpha", WindowsServiceState::Stopped),
        ];
        sort_rows(&mut rows, ServiceColumn::Name, false);
        assert_eq!(&*rows[0].name, "Alpha");
        assert_eq!(&*rows[1].name, "beta");
    }

    #[test]
    fn the_name_column_sorts_by_the_name_it_shows() {
        let mut rows = vec![
            ServiceRow {
                display_name: "Windows Audio".into(),
                ..row("AudioSrv", WindowsServiceState::Running)
            },
            ServiceRow {
                display_name: "DHCP Client".into(),
                ..row("Dhcp", WindowsServiceState::Running)
            },
        ];
        sort_rows(&mut rows, ServiceColumn::Name, false);
        assert_eq!(&*rows[0].display_name, "DHCP Client");
        assert_eq!(&*rows[1].display_name, "Windows Audio");
    }

    #[test]
    fn sorts_by_status_when_requested() {
        let mut rows = vec![
            row("a", WindowsServiceState::Running),
            row("b", WindowsServiceState::Paused),
        ];
        sort_rows(&mut rows, ServiceColumn::Status, false);
        assert_eq!(rows[0].state, WindowsServiceState::Paused);
    }

    #[test]
    fn controls_follow_status_and_keep_old_peer_support() {
        let mut service = row("sample", WindowsServiceState::Stopped);
        assert!(service.can_start());
        assert!(!service.can_stop());
        service.startup_type = app_contracts::features::agents::WindowsServiceStartType::Disabled;
        assert!(!service.can_start());
        service.state = WindowsServiceState::Running;
        assert!(service.can_stop());
        service.service_type = 0x10;
        assert!(!service.can_stop());
        service.accepted_controls = 1;
        assert!(service.can_stop());
        service.state = WindowsServiceState::StopPending;
        assert!(!service.can_start());
        assert!(!service.can_stop());
    }
}
