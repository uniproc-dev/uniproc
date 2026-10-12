use std::rc::Rc;
use std::sync::Arc;

use app_contracts::features::agents::{
    ActionFailure, ActionOutcome, WindowsAction, WindowsReportMessage, WindowsServiceStats,
};
use app_contracts::features::services::{
    Command, Deselect, DismissFailure, Select, ServiceActionKind, ServiceColumn, ServiceRow, ServicesMsg,
    ServicesState, Sort,
};
use guinea::prelude::*;

use crate::features::agents::actions;

pub struct Acted {
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
}

impl ServicesActor {
    pub fn new(push: Push<ServicesState>) -> Self {
        Self {
            push,
            rows: Rc::from(Vec::new()),
            sort_column: ServiceColumn::Name,
            descending: false,
            selected: None,
        }
    }

    fn publish_rows(&self) {
        self.push.send(ServicesMsg::SetRows {
            rows: self.rows.clone(),
        });
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
            Command,
            WindowsReportMessage,
            Acted,
            DismissFailure,
        }
    }
}

#[handler]
fn sort(this: &mut ServicesActor, Sort(column): Sort) {
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
fn select(this: &mut ServicesActor, Select(name): Select) {
    this.selected = Some(name.clone());
    this.push.send(ServicesMsg::SetSelected(Some(name)));
}

#[handler]
fn deselect(this: &mut ServicesActor, _msg: Deselect) {
    this.selected = None;
    this.push.send(ServicesMsg::SetSelected(None));
}

#[handler]
fn command(this: &mut ServicesActor, Command(kind): Command, cx: Cx) {
    let Some(name) = this.selected.clone() else {
        return;
    };
    let target = this
        .rows
        .iter()
        .find(|row| *row.name == *name)
        .map_or_else(|| Arc::from(name.as_str()), |row| row.display_name.clone());
    let action = match kind {
        ServiceActionKind::Start => WindowsAction::ServiceStart { name },
        ServiceActionKind::Stop => WindowsAction::ServiceStop { name },
        ServiceActionKind::Pause => WindowsAction::ServicePause { name },
        ServiceActionKind::Resume => WindowsAction::ServiceResume { name },
        ServiceActionKind::Restart => WindowsAction::ServiceRestart { name },
    };
    cx.spawn_bg(async move {
        let outcome = actions::request(action.clone()).await;
        Acted { action, target, outcome }
    });
}

#[handler]
fn on_windows_report(this: &mut ServicesActor, msg: WindowsReportMessage) {
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
fn on_acted(this: &mut ServicesActor, Acted { action, target, outcome }: Acted) {
    if outcome == ActionOutcome::Done {
        return;
    }
    tracing::warn!(?action, ?outcome, "the action did not go through");
    this.push.send(ServicesMsg::SetFailure(Some(ActionFailure { action, target, outcome })));
}

#[handler]
fn dismiss_failure(this: &mut ServicesActor, _msg: DismissFailure) {
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
}
