use std::rc::Rc;

use guinea::prelude::*;

use super::messages::SystemMsg;
use super::model::SystemTool;

#[derive(Clone, PartialEq, Debug)]
pub struct SystemState {
    pub missing: Load<Rc<[SystemTool]>>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self { missing: Load::Loading }
    }
}

impl SystemState {
    pub fn found(&self, tool: SystemTool) -> Option<bool> {
        self.missing.ready().map(|missing| !missing.contains(&tool))
    }
}

#[reducer]
fn system(this: &mut SystemState, SystemMsg::Set { missing }: SystemMsg) {
    this.missing = Load::Ready(missing);
}
