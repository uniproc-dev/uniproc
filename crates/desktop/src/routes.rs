use guinea::prelude::*;

use crate::layouts::{ShellLayout, TabsLayout};
use crate::pages::{Processes, Services, Settings, Wsl};

routes! {
    Route {
        layout(TabsLayout) restorable {
            layout(ShellLayout) {
                page(Processes)
                page(Services)
                page(Wsl)
                page(Settings)
            }
        }
    }
}
