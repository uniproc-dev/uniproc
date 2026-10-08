mod actor;
mod elevation;
mod in_process;
mod install;

pub use elevation::{wait_for_the_copy_it_replaces, Elevation, RelaunchError};
pub use in_process::{InProcessAgent, InProcessStart, InProcessStartError};
pub use install::{AgentLinkDeps, AgentLinkFeature};
