use std::sync::Arc;

use guinea::prelude::Event;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub struct ProcessInstance {
    pub pid: u32,
    pub sequence: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ScheduledTask {
    pub name: Arc<str>,
    pub path: Arc<str>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ProcessCame {
    pub instance: ProcessInstance,
    pub parent: ProcessInstance,
    pub at: u64,
    pub image_path: Arc<str>,
    pub command_line: Arc<str>,
    pub working_dir: Arc<str>,
    pub user: Arc<str>,
    pub session_id: u32,
    pub elevated: Option<bool>,
    pub scheduled_task: Option<ScheduledTask>,
    pub parent_services: Arc<[Arc<str>]>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ProcessWent {
    pub instance: ProcessInstance,
    pub at: u64,
    pub image_path: Arc<str>,
    pub image_name: Arc<str>,
    pub started_at: Option<u64>,
    pub exit_code: u32,
    pub cpu_cycles: u64,
    pub io_read_ops: u64,
    pub io_write_ops: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub peak_commit_bytes: u64,
    pub handles: u32,
    pub hard_faults: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum ProcessEvent {
    Came(ProcessCame),
    Went(ProcessWent),
}

impl ProcessEvent {
    pub fn at(&self) -> u64 {
        match self {
            Self::Came(came) => came.at,
            Self::Went(went) => went.at,
        }
    }

    pub fn instance(&self) -> ProcessInstance {
        match self {
            Self::Came(came) => came.instance,
            Self::Went(went) => went.instance,
        }
    }
}

#[derive(Clone, Debug, Default, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsProcessEvents {
    pub history_from: Option<u64>,
    pub events: Arc<[ProcessEvent]>,
    pub lost: u32,
}
