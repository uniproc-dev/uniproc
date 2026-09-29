use std::rc::Rc;

use guinea::prelude::*;

use super::messages::SystemMsg;
use super::model::SystemTool;

#[derive(Clone, PartialEq, Debug)]
pub struct SystemState {
    pub missing: Load<Rc<[SystemTool]>>,
    pub pinned: Rc<[SystemTool]>,
    pub frequent: Rc<[SystemTool]>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            missing: Load::Loading,
            pinned: Rc::from([]),
            frequent: Rc::from([]),
        }
    }
}

impl SystemState {
    pub fn found(&self, tool: SystemTool) -> Option<bool> {
        self.missing.ready().map(|missing| !missing.contains(&tool))
    }

    pub fn is_pinned(&self, tool: SystemTool) -> bool {
        self.pinned.contains(&tool)
    }

    pub fn favourites(&self) -> impl Iterator<Item = SystemTool> + '_ {
        self.pinned.iter().chain(self.frequent.iter()).copied()
    }
}

#[reducer]
fn system(this: &mut SystemState, msg: SystemMsg) {
    match msg {
        SystemMsg::Missing(missing) => this.missing = Load::Ready(missing),
        SystemMsg::Favourites { pinned, frequent } => {
            this.pinned = pinned;
            this.frequent = frequent;
        }
    }
}
