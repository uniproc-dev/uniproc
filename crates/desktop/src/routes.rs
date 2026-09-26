use guinea::prelude::*;

use crate::layouts::{ShellLayout, TabsLayout};
use crate::pages::{Processes, Services, Wsl};

routes! {
    Route {
        layout(TabsLayout) restorable {
            layout(ShellLayout) {
                page(Processes)
                page(Services)
                page(Wsl)
            }
        }
    }
}
