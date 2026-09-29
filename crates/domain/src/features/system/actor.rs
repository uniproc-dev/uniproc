use std::rc::Rc;

use app_contracts::features::system::{GetTool, OpenTool, SystemMsg, SystemState, SystemTool};
use guinea::prelude::*;

use super::install::SystemDeps;
use super::tools;

#[derive(Debug)]
pub struct SystemActor {
    ui_port: Push<SystemState>,
    deps: SystemDeps,
}

impl SystemActor {
    pub fn new(ui_port: Push<SystemState>, deps: SystemDeps) -> Self {
        Self { ui_port, deps }
    }
}

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Rescan;

struct Scanned(Vec<SystemTool>);

actor! {
    SystemActor {
        handlers { Rescan, Scanned, OpenTool, GetTool }
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
    this.ui_port.send(SystemMsg::Set {
        missing: Rc::from(missing),
    });
}

#[handler]
async fn open(ctx: AsyncContext<SystemActor>, OpenTool(tool): OpenTool) {
    let Some(deps) = ctx.apply(|this, _| this.deps).await else {
        return;
    };
    match tools::resolve(tool, deps.locate) {
        Some(launch) => (deps.launch)(launch),
        None => {
            tracing::info!(?tool, "system tool is gone, looking again");
            ctx.send(Rescan);
        }
    }
}

#[handler]
fn get(this: &SystemActor, GetTool(tool): GetTool) {
    if let Some(page) = tools::download(tool) {
        (this.deps.launch)(page);
    }
}
