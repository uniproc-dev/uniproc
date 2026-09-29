use guinea::prelude::*;

use crate::layouts::{ProcessesArea, ShellLayout, SystemArea, TabsLayout};
use crate::pages::{Processes, ProcessesSettings, Services, Settings, System, SystemTools, Wsl};

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
                layout(SystemArea) {
                    page(System)
                    page(SystemTools)
                }
                page(Settings)
            }
        }
    }
}
