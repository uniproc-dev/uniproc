use guinea::prelude::*;

use super::messages::SidebarMsg;

#[derive(Clone, PartialEq, Debug)]
pub struct SidebarState {
    pub open: bool,
    pub width: u64,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            open: true,
            width: 260,
        }
    }
}

#[reducer]
fn sidebar(this: &mut SidebarState, SidebarMsg::Set { open, width }: SidebarMsg) {
    this.open = open;
    this.width = width;
}
