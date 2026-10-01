use app_contracts::features::agents::{
    AgentStateRequest, RemoteScanResult, WindowsReportMessage,
};
use app_contracts::features::processes::ProcessesState;
use app_contracts::features::window::PressedAway;
use guinea::prelude::*;

use super::actor::ProcessesActor;
use super::shell::{self, ShellRequest};
use super::windows_scan::{self, AppWindows};

#[derive(Clone, Copy)]
pub struct ProcessesDeps {
    pub windows: fn() -> AppWindows,
    pub shell: fn(ShellRequest),
}

impl Default for ProcessesDeps {
    fn default() -> Self {
        Self {
            windows: windows_scan::app_windows,
            shell: shell::run,
        }
    }
}

feature! {
    pub ProcessesFeature {
        exports { ProcessesState }
    }
}

#[installs]
fn processes(cx: &FeatureInitContext, deps: &ProcessesDeps) -> anyhow::Result<ProcessesFeature> {
    let deps = *deps;
    let (processes, addr) = cx
        .state::<ProcessesState>()
        .driven_by(move |port| ProcessesActor::new(port, deps.windows, deps.shell));
    addr.subscribe_on::<WindowsReportMessage>(Bus::Global);
    addr.subscribe_on::<RemoteScanResult>(Bus::Global);
    addr.subscribe_on::<PressedAway>(Bus::Global);

    GlobalEventBus::publish(AgentStateRequest);

    Ok(ProcessesFeature(processes))
}
