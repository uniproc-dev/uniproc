use app_contracts::features::agents::{AgentStateRequest, ScanTick, WindowsReportMessage};
use app_contracts::features::processes::ProcessesState;
use guinea::prelude::*;

use super::actor::ProcessesActor;
use super::windows_scan::{self, AppWindows};

#[derive(Clone, Copy)]
pub struct ProcessesParams {
    pub windows: fn() -> AppWindows,
}

impl Default for ProcessesParams {
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
fn processes(cx: &FeatureInitContext, params: &ProcessesParams) -> anyhow::Result<ProcessesFeature> {
    let windows = params.windows;
    let (processes, addr) = cx
        .state::<ProcessesState>()
        .driven_by(move |port| ProcessesActor::new(port, windows));
    addr.subscribe_on::<WindowsReportMessage>(Bus::Global);

    GlobalEventBus::publish(AgentStateRequest);
    GlobalEventBus::publish(ScanTick);

    Ok(ProcessesFeature(processes))
}
