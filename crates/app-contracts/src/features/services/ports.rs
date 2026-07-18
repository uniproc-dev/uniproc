use forsl::native_windows::slint_factory::SlintWindowRegistry;
use slint::SharedString;
use std::fmt::Debug;

use super::model::ServiceEntryVm;

pub trait ServicesWindowRegister {
    fn register(&self, registry: &SlintWindowRegistry);
}

#[derive(Clone, Debug)]
pub enum UiServiceDetailsPortMsg {
    SetSelectedServiceDetails(ServiceEntryVm),
    SetActiveButtons {
        start_button_active: bool,
        stop_button_active: bool,
        restart_button_active: bool,
    },
}

pub trait UiServiceDetailsPort {
    fn send(&self, msg: UiServiceDetailsPortMsg);
}

#[derive(Clone, Debug)]
pub enum UiServicesPortMsg {
    SetColumnWidths(Vec<(SharedString, u64)>),
    SetServiceRowsWindow { total_rows: usize, start: usize, rows: Vec<ServiceEntryVm> },
    SetCurrentSort(SharedString),
    SetCurrentSortDescending(bool),
    SetTotalServicesCount(usize),
}

pub trait UiServicesPort: Debug + UiServiceDetailsPort + 'static {
    fn send(&self, msg: UiServicesPortMsg);
}
