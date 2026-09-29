use guinea::prelude::*;

use crate::layouts::{ProcessesArea, ShellLayout, TabsLayout};
use crate::pages::{Processes, ProcessesSettings, Services, Settings, System, Wsl};

routes! {
    Route {
        layout(TabsLayout) restorable {
            layout(ShellLayout) {
                layout(ProcessesArea) {
                    page(Processes)
                    page(ProcessesSettings)
                }
                page(Services)
                page(Wsl)
                page(System)
                page(Settings)
            }
        }
    }
}
