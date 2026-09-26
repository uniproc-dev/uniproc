use app_contracts::features::agents::{AgentStateRequest, ScanTick, WindowsReportMessage};
use app_contracts::features::processes::ProcessesState;
use guinea::prelude::*;

use super::actor::ProcessesActor;
use super::windows_scan::{self, AppWindows};

#[derive(Clone, Copy)]
pub struct ProcessesDeps {
    pub windows: fn() -> AppWindows,
}

impl Default for ProcessesDeps {
    fn default() -> Self {
        Self {
            windows: windows_scan::app_windows,
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
    let windows = deps.windows;
    let (processes, addr) = cx
        .state::<ProcessesState>()
        .driven_by(move |port| ProcessesActor::new(port, windows));
    addr.subscribe_on::<WindowsReportMessage>(Bus::Global);

    GlobalEventBus::publish(AgentStateRequest);
    GlobalEventBus::publish(ScanTick);

    Ok(ProcessesFeature(processes))
}
