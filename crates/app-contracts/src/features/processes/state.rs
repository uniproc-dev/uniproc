use crate::features::agents::AgentConnectionState;
use guinea::prelude::*;
use std::rc::Rc;

use super::messages::ProcessesMsg;
use super::model::{MachineSummary, ProcessColumn, ProcessRow, WslEnvironment, WslProcess};

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessesState {
    pub rows: Load<Rc<[ProcessRow]>>,
    pub wsl: Rc<[WslEnvironment]>,
    pub machine_summary: Load<MachineSummary>,
    pub selected: Option<u32>,
    pub selected_linux: Option<u32>,
    pub sort_column: ProcessColumn,
    pub descending: bool,
    pub agent_state: AgentConnectionState,
}

impl Default for ProcessesState {
    fn default() -> Self {
        Self {
            rows: Load::Loading,
            wsl: Rc::from(Vec::new()),
            machine_summary: Load::Loading,
            selected: None,
            selected_linux: None,
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

    pub fn linux_process(&self, global_pid: u32) -> Option<&WslProcess> {
        self.wsl
            .iter()
            .flat_map(|environment| environment.processes.iter())
            .find(|process| process.global_pid == global_pid)
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
            ProcessesMsg::SetWsl(environments) => {
                self.wsl = environments;
            }
            ProcessesMsg::SetSelected(pid) => {
                self.selected = pid;
            }
            ProcessesMsg::SetSelectedLinux(global_pid) => {
                self.selected_linux = global_pid;
            }
            ProcessesMsg::SetSort { column, descending } => {
                self.sort_column = column;
                self.descending = descending;
            }
        }
    }
}
