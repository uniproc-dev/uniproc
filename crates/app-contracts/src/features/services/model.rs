use std::sync::Arc;

use crate::features::agents::WindowsServiceState;

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
pub enum ServiceColumn {
    Name,
    Status,
    Pid,
    Group,
    Description,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServiceRow {
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub pid: u32,
    pub state: WindowsServiceState,
    pub group: Arc<str>,
    pub description: Arc<str>,
    pub image_path: Arc<str>,
}

impl ServiceRow {
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum ServiceActionKind {
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
}
