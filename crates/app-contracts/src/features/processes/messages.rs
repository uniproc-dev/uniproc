use crate::features::agents::{ActionFailure, AgentConnectionState, ProcessPriority};
use serde::Deserialize;
use std::rc::Rc;

use super::model::{MachineSummary, ProcessColumn, ProcessRow, WslEnvironment};

#[derive(Clone)]
pub enum ProcessesMsg {
    SetRows {
        rows: Rc<[ProcessRow]>,
        machine: MachineSummary,
        agent_state: AgentConnectionState,
    },
    SetWsl(Rc<[WslEnvironment]>),
    SetSelected(Option<u32>),
    SetSelectedLinux(Option<u32>),
    SetSort {
        column: ProcessColumn,
        descending: bool,
    },
    SetFailure(Option<ActionFailure>),
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct DismissFailure;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Sort(pub ProcessColumn);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Select(pub u32);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SelectLinux(pub u32);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Deselect;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Terminate;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum GroupCommand {
    End,
    Suspend,
    Resume,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RunGroupCommand {
    pub pids: Vec<u32>,
    pub command: GroupCommand,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RunNewTask;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ProcessCommand {
    Suspend,
    Resume,
    Priority(ProcessPriority),
    OpenFileLocation,
    Properties,
    SearchOnline,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RunProcessCommand(pub ProcessCommand);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RunImageCommand {
    pub command: ProcessCommand,
    pub exe_path: String,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum WindowCommand {
    SwitchTo,
    Minimize,
    Maximize,
    Close,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RunWindowCommand {
    pub handle: isize,
    pub command: WindowCommand,
}
