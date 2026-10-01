use guinea::prelude::*;

use crate::layouts::{ProcessesArea, ShellLayout, SystemArea};
use crate::pages::{Processes, ProcessesSettings, Services, Settings, System, SystemTools, Wsl};

routes! {
    Route {
        layout(ShellLayout) restorable {
            layout(ProcessesArea) {
                page(Processes)
                page(ProcessesSettings)
            }
            page(Services)
            page(Wsl)
            layout(SystemArea) {
                page(System)
                page(SystemTools)
            }
            page(Settings)
        }
    }
}
