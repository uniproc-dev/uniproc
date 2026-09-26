use app_contracts::features::sidebar::{SetOpen, SetWidth, SidebarMsg, SidebarState, Toggle};
use guinea::prelude::*;

use super::settings::SidebarSettings;

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Refresh;

#[derive(Debug)]
pub struct SidebarActor {
    ui_port: Push<SidebarState>,
    settings: SidebarSettings,
}

impl SidebarActor {
    pub fn new(ui_port: Push<SidebarState>, settings: SidebarSettings) -> Self {
        Self { ui_port, settings }
    }

    fn publish(&self) {
        let open = self.settings.open().get();
        let width = self.settings.width().get();
        tracing::debug!(open, width, "sidebar publish");
        self.ui_port.send(SidebarMsg::Set { open, width });
    }
}

actor! {
    SidebarActor {
        handlers { Toggle, SetOpen, SetWidth, Refresh }
    }
}

#[handler]
fn toggle(this: &mut SidebarActor, _ctx: Context<SidebarActor, Toggle>) {
    let open = !this.settings.open().get();
    let _ = this.settings.open().set(open);
    this.publish();
}

#[handler]
fn set_open(this: &mut SidebarActor, ctx: Context<SidebarActor, SetOpen>) {
    let _ = this.settings.open().set(ctx.msg.0);
    this.publish();
}

#[handler]
fn set_width(this: &mut SidebarActor, ctx: Context<SidebarActor, SetWidth>) {
    let _ = this.settings.width().set(ctx.msg.0);
    this.publish();
}

#[handler]
fn refresh(this: &SidebarActor, _ctx: Context<SidebarActor, Refresh>) {
    this.publish();
}
