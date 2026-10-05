use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use app_contracts::features::system::{
    ForgetTool, GetTool, OpenTool, PinTool, SystemMsg, SystemState, SystemTool,
};
use app_contracts::OrWarn;
use guinea::prelude::*;

use super::favourites;
use super::install::SystemDeps;
use super::settings::{SystemSettings, ToolUse};
use super::tools;

#[derive(Debug)]
pub struct SystemActor {
    ui_port: Push<SystemState>,
    deps: SystemDeps,
    settings: SystemSettings,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or_default()
}

pub(super) fn favourites(settings: &SystemSettings) -> (Rc<[SystemTool]>, Rc<[SystemTool]>) {
    let pinned = favourites::pinned(settings.pinned().entries());
    let frequent = favourites::frequent(settings.uses().entries(), &pinned, now_ms());
    (Rc::from(pinned), Rc::from(frequent))
}

impl SystemActor {
    pub fn new(ui_port: Push<SystemState>, deps: SystemDeps, settings: SystemSettings) -> Self {
        Self {
            ui_port,
            deps,
            settings,
        }
    }

    fn publish_favourites(&self) {
        let (pinned, frequent) = favourites(&self.settings);
        self.ui_port.send(SystemMsg::Favourites { pinned, frequent });
    }
}

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Rescan;

struct Scanned(Vec<SystemTool>);

enum Opened {
    Launched(SystemTool),
    Gone(SystemTool),
}

actor! {
    SystemActor {
        handlers { Rescan, Scanned, OpenTool, Opened, GetTool, PinTool, ForgetTool }
    }
}

#[handler]
fn rescan(this: &mut SystemActor, _: Rescan, cx: Cx) {
    let locate = this.deps.locate;
    cx.spawn_bg(async move { Scanned(tools::missing(locate)) });
}

#[handler]
fn on_scanned(this: &mut SystemActor, Scanned(missing): Scanned) {
    this.ui_port.send(SystemMsg::Missing(Rc::from(missing)));
}

#[handler]
fn open(this: &mut SystemActor, OpenTool(tool): OpenTool, cx: Cx) {
    let deps = this.deps;
    cx.spawn_bg(async move {
        match tools::resolve(tool, deps.locate) {
            Some(launch) => {
                (deps.launch)(launch);
                Opened::Launched(tool)
            }
            None => Opened::Gone(tool),
        }
    });
}

#[handler]
fn on_opened(this: &mut SystemActor, opened: Opened, cx: Cx) {
    let tool = match opened {
        Opened::Launched(tool) => tool,
        Opened::Gone(tool) => {
            tracing::info!(?tool, "system tool is gone, looking again");
            let locate = this.deps.locate;
            cx.spawn_bg(async move { Scanned(tools::missing(locate)) });
            return;
        }
    };
    let uses = this.settings.uses();
    let before = uses.get(tool.id()).unwrap_or_default();
    let now = ToolUse {
        count: before.count.saturating_add(1),
        last_ms: now_ms(),
    };
    uses.insert(tool.id().to_string(), &now)
        .or_warn(format_args!("could not count a use of {tool:?}"));
    this.publish_favourites();
}

#[handler]
fn get(this: &SystemActor, GetTool(tool): GetTool) {
    if let Some(page) = tools::download(tool) {
        (this.deps.launch)(page);
    }
}

#[handler]
fn pin(this: &mut SystemActor, PinTool(tool, pinned): PinTool) {
    let pins = this.settings.pinned();
    if pinned {
        let next = pins.entries().map(|(_, order)| order + 1).max().unwrap_or_default();
        pins.insert(tool.id().to_string(), &next).or_warn(format_args!("could not pin {tool:?}"));
    } else {
        pins.remove(tool.id()).or_warn(format_args!("could not unpin {tool:?}"));
    }
    this.publish_favourites();
}

#[handler]
fn forget(this: &mut SystemActor, ForgetTool(tool): ForgetTool) {
    this.settings.pinned().remove(tool.id()).or_warn(format_args!("could not unpin {tool:?}"));
    this.settings.uses().remove(tool.id()).or_warn(format_args!("could not forget the uses of {tool:?}"));
    this.publish_favourites();
}
