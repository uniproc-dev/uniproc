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
fn toggle(this: &mut SidebarActor, _msg: Toggle) {
    let open = !this.settings.open().get();
    this.settings.open().set(open);
    this.publish();
}

#[handler]
fn set_open(this: &mut SidebarActor, SetOpen(open): SetOpen) {
    this.settings.open().set(open);
    this.publish();
}

#[handler]
fn set_width(this: &mut SidebarActor, SetWidth(width): SetWidth) {
    this.settings.width().set(width);
    this.publish();
}

#[handler]
fn refresh(this: &SidebarActor, _msg: Refresh) {
    this.publish();
}
