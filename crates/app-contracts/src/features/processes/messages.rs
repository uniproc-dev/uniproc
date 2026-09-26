use crate::features::agents::AgentConnectionState;
use serde::Deserialize;
use std::rc::Rc;

use super::model::{MachineSummary, ProcessColumn, ProcessRow};

#[derive(Clone)]
pub enum ProcessesMsg {
    SetRows {
        rows: Rc<[ProcessRow]>,
        machine: MachineSummary,
        agent_state: AgentConnectionState,
    },
    SetSelected(Option<u32>),
    SetSort {
        column: ProcessColumn,
        descending: bool,
    },
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Sort(pub ProcessColumn);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Select(pub u32);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Deselect;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Terminate;
