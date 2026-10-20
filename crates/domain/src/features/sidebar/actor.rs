use app_contracts::features::sidebar::{SetOpen, SetWidth, SidebarMsg, SidebarState, Toggle};
use guinea::prelude::*;

use super::settings::SidebarSettings;

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Refresh;

#[derive(Debug)]
pub struct SidebarActor {
    push: Push<SidebarState>,
    settings: SidebarSettings,
}

impl SidebarActor {
    pub fn new(push: Push<SidebarState>, settings: SidebarSettings) -> Self {
        Self { push, settings }
    }

    fn publish(&self) {
        let open = self.settings.open().get();
        let width = self.settings.width().get();
        tracing::debug!(open, width, "sidebar publish");
        self.push.send(SidebarMsg::Set { open, width });
    }
}

actor! {
    SidebarActor {
        handlers {
            Toggle,
            SetOpen,
            SetWidth,
            Refresh,
        }
    }
}

#[handler]
fn toggle(In(this, _, _): In<SidebarActor, Toggle>) {
    let open = !this.settings.open().get();
    this.settings.open().set(open);
    this.publish();
}

#[handler]
fn set_open(In(this, SetOpen(open), _): In<SidebarActor, SetOpen>) {
    this.settings.open().set(open);
    this.publish();
}

#[handler]
fn set_width(In(this, SetWidth(width), _): In<SidebarActor, SetWidth>) {
    this.settings.width().set(width);
    this.publish();
}

#[handler]
fn refresh(In(this, _, _): In<SidebarActor, Refresh>) {
    this.publish();
}
