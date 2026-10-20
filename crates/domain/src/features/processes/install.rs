use app_contracts::features::processes::ProcessesState;
use guinea::prelude::*;

use super::actor::{ProcessesActor, Woke};
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
fn processes(cx: &Install<ProcessesDeps>) -> anyhow::Result<ProcessesFeature> {
    let deps = *cx.params();
    let (processes, addr) = cx
        .state::<ProcessesState>()
        .driven_by(move |port| ProcessesActor::new(port, deps.windows, deps.shell));

    addr.send(Woke);
    cx.on_wake(move || {
        addr.send(Woke);
    });

    Ok(ProcessesFeature(processes))
}
