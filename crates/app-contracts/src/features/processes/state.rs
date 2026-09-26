use crate::features::agents::AgentConnectionState;
use guinea::prelude::*;
use std::rc::Rc;

use super::messages::ProcessesMsg;
use super::model::{MachineSummary, ProcessColumn, ProcessRow};

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessesState {
    pub rows: Load<Rc<[ProcessRow]>>,
    pub machine_summary: Load<MachineSummary>,
    pub selected: Option<u32>,
    pub sort_column: ProcessColumn,
    pub descending: bool,
    pub agent_state: AgentConnectionState,
}

impl Default for ProcessesState {
    fn default() -> Self {
        Self {
            rows: Load::Loading,
            machine_summary: Load::Loading,
            selected: None,
            agent_state: AgentConnectionState::Disconnected,
            sort_column: ProcessColumn::Cpu,
            descending: true,
        }
    }
}

impl ProcessesState {
    pub fn rows(&self) -> &[ProcessRow] {
        self.rows.ready().map(|r| r.as_ref()).unwrap_or(&[])
    }

    pub fn total(&self) -> usize {
        self.rows().len()
    }

    pub fn machine_summary(&self) -> Option<&MachineSummary> {
        self.machine_summary.ready()
    }
}

impl Reducer for ProcessesState {
    type Update = ProcessesMsg;

    fn reduce(&mut self, update: ProcessesMsg) {
        match update {
            ProcessesMsg::SetRows { rows, machine, agent_state } => {
                self.rows = Load::Ready(rows);
                self.machine_summary = Load::Ready(machine);
                self.agent_state = agent_state;
            }
            ProcessesMsg::SetSelected(pid) => {
                self.selected = pid;
            }
            ProcessesMsg::SetSort { column, descending } => {
                self.sort_column = column;
                self.descending = descending;
            }
        }
    }
}
