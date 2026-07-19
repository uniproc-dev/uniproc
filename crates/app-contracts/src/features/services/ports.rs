use forsl_macros::port;
use slint::SharedString;
use std::fmt::Debug;

use super::model::ServiceEntryVm;

#[derive(Clone, Debug)]
pub enum UiServiceDetailsPortMsg {
    SetSelectedServiceDetails(ServiceEntryVm),
    SetActiveButtons {
        start_button_active: bool,
        stop_button_active: bool,
        restart_button_active: bool,
    },
}

#[port]
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

#[port]
pub trait UiServicesPort: Debug + UiServiceDetailsPort + 'static {
    fn send(&self, msg: UiServicesPortMsg);
}
