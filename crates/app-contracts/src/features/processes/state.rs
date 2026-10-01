use crate::features::agents::{ActionFailure, AgentConnectionState};
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
    pub failure: Option<ActionFailure>,
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
            failure: None,
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

    pub fn linux_process(&self, global_pid: u32) -> Option<(&WslEnvironment, &WslProcess)> {
        self.wsl.iter().find_map(|environment| {
            let process = environment.processes.iter().find(|p| p.global_pid == global_pid)?;
            Some((environment, process))
        })
    }
}

#[reducer]
fn processes(this: &mut ProcessesState, update: ProcessesMsg) {
    match update {
        ProcessesMsg::SetRows { rows, machine, agent_state } => {
            this.rows = Load::Ready(rows);
            this.machine_summary = Load::Ready(machine);
            this.agent_state = agent_state;
        }
        ProcessesMsg::SetWsl(environments) => this.wsl = environments,
        ProcessesMsg::SetSelected(pid) => this.selected = pid,
        ProcessesMsg::SetSelectedLinux(global_pid) => this.selected_linux = global_pid,
        ProcessesMsg::SetSort { column, descending } => {
            this.sort_column = column;
            this.descending = descending;
        }
        ProcessesMsg::SetFailure(failure) => this.failure = failure,
    }
}
