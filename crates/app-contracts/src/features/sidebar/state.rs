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

impl Reducer for SidebarState {
    type Update = SidebarMsg;

    fn reduce(&mut self, update: SidebarMsg) {
        match update {
            SidebarMsg::Set { open, width } => {
                self.open = open;
                self.width = width;
            }
        }
    }
}
