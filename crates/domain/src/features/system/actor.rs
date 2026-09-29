use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use app_contracts::features::system::{GetTool, OpenTool, PinTool, SystemMsg, SystemState, SystemTool};
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

struct Used(SystemTool);

actor! {
    SystemActor {
        handlers { Rescan, Scanned, OpenTool, Used, GetTool, PinTool }
    }
}

#[handler]
async fn rescan(ctx: AsyncContext<SystemActor>, _: Rescan) {
    let Some(locate) = ctx.apply(|this, _| this.deps.locate).await else {
        return;
    };
    ctx.send(Scanned(tools::missing(locate)));
}

#[handler]
fn on_scanned(this: &mut SystemActor, Scanned(missing): Scanned) {
    this.ui_port.send(SystemMsg::Missing(Rc::from(missing)));
}

#[handler]
async fn open(ctx: AsyncContext<SystemActor>, OpenTool(tool): OpenTool) {
    let Some(deps) = ctx.apply(|this, _| this.deps).await else {
        return;
    };
    match tools::resolve(tool, deps.locate) {
        Some(launch) => {
            (deps.launch)(launch);
            ctx.send(Used(tool));
        }
        None => {
            tracing::info!(?tool, "system tool is gone, looking again");
            ctx.send(Rescan);
        }
    }
}

#[handler]
fn on_used(this: &mut SystemActor, Used(tool): Used) {
    let uses = this.settings.uses();
    let before = uses.get(tool.id()).unwrap_or_default();
    let now = ToolUse {
        count: before.count.saturating_add(1),
        last_ms: now_ms(),
    };
    if let Err(err) = uses.insert(tool.id().to_string(), &now) {
        tracing::warn!(?err, ?tool, "could not count a system tool use");
    }
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
    let kept = if pinned {
        let next = pins.entries().map(|(_, order)| order + 1).max().unwrap_or_default();
        pins.insert(tool.id().to_string(), &next).map(|_| ())
    } else {
        pins.remove(tool.id()).map(|_| ())
    };
    if let Err(err) = kept {
        tracing::warn!(?err, ?tool, pinned, "could not keep a system tool pin");
    }
    this.publish_favourites();
}
