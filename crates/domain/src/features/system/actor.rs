use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use app_contracts::features::system::{
    ForgetTool, GetTool, OpenTool, PinTool, SystemMsg, SystemState, SystemTool,
};
use guinea::prelude::*;

use super::favourites;
use super::install::SystemDeps;
use super::settings::{SystemSettings, ToolUse};
use super::tools;

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Rescan;

enum Opened {
    Launched(SystemTool),
    Gone(SystemTool),
}

#[derive(Debug)]
pub struct SystemActor {
    push: Push<SystemState>,
    deps: SystemDeps,
    settings: SystemSettings,
}

impl SystemActor {
    pub fn new(push: Push<SystemState>, deps: SystemDeps, settings: SystemSettings) -> Self {
        Self {
            push,
            deps,
            settings,
        }
    }

    fn publish_favourites(&self) {
        let (pinned, frequent) = favourites(&self.settings);
        self.push.send(SystemMsg::Favourites { pinned, frequent });
    }
}

actor! {
    SystemActor {
        handlers {
            Rescan,
            OpenTool => { task Opened }
            Opened,
            GetTool,
            PinTool,
            ForgetTool,
        }
    }
}

#[handler]
fn rescan(In(this, _, cx): In<SystemActor, Rescan>) {
    let locate = this.deps.locate;
    cx.task()
        .pushing_with(this.push, |missing: Vec<SystemTool>| SystemMsg::Missing(Rc::from(missing)))
        .spawn(async move { tools::missing(locate) });
}

#[handler]
fn open(In(this, OpenTool(tool), cx): In<SystemActor, OpenTool>) {
    let deps = this.deps;
    cx.task().spawn(async move {
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
fn on_opened(In(this, opened, cx): In<SystemActor, Opened>) {
    let tool = match opened {
        Opened::Launched(tool) => tool,
        Opened::Gone(tool) => {
            tracing::info!(?tool, "system tool is gone, looking again");
            let locate = this.deps.locate;
            cx.task()
                .pushing_with(this.push, |missing: Vec<SystemTool>| SystemMsg::Missing(Rc::from(missing)))
                .spawn(async move { tools::missing(locate) });
            return;
        }
    };
    let uses = this.settings.uses();
    let before = uses.get(tool.id()).unwrap_or_default();
    let now = ToolUse {
        count: before.count.saturating_add(1),
        last_ms: now_ms(),
    };
    if let Err(err) = uses.insert(tool.id().to_string(), &now) {
        tracing::warn!(?err, ?tool, "could not count a use of the tool");
    }
    this.publish_favourites();
}

#[handler]
fn get(In(this, GetTool(tool), _): In<SystemActor, GetTool>) {
    if let Some(page) = tools::download(tool) {
        (this.deps.launch)(page);
    }
}

#[handler]
fn pin(In(this, PinTool(tool, pinned), _): In<SystemActor, PinTool>) {
    let pins = this.settings.pinned();
    if pinned {
        let next = pins.entries().map(|(_, order)| order + 1).max().unwrap_or_default();
        if let Err(err) = pins.insert(tool.id().to_string(), &next) {
            tracing::warn!(?err, ?tool, "could not pin the tool");
        }
    } else {
        if let Err(err) = pins.remove(tool.id()) {
            tracing::warn!(?err, ?tool, "could not unpin the tool");
        }
    }
    this.publish_favourites();
}

#[handler]
fn forget(In(this, ForgetTool(tool), _): In<SystemActor, ForgetTool>) {
    if let Err(err) = this.settings.pinned().remove(tool.id()) {
        tracing::warn!(?err, ?tool, "could not unpin the tool");
    }
    if let Err(err) = this.settings.uses().remove(tool.id()) {
        tracing::warn!(?err, ?tool, "could not forget the uses of the tool");
    }
    this.publish_favourites();
}

pub(super) fn favourites(settings: &SystemSettings) -> (Rc<[SystemTool]>, Rc<[SystemTool]>) {
    let pinned = favourites::pinned(settings.pinned().entries());
    let frequent = favourites::frequent(settings.uses().entries(), &pinned, now_ms());
    (Rc::from(pinned), Rc::from(frequent))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or_default()
}
