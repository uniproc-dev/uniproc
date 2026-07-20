use forsl_macros::port;
use slint::SharedString;
use std::fmt::Debug;

use super::model::{FieldDefDto, FieldMetadata, ProcessEntryVm};

#[derive(Clone, Debug, PartialEq)]
pub enum UiProcessesPortMsg {
    SetColumnWidths(Vec<(SharedString, u64)>),
    SetColumnMetadata(Vec<FieldMetadata>),
    SetProcessRowsWindow { total_rows: usize, start: usize, rows: Vec<ProcessEntryVm> },
    SetColumnDefs(Vec<FieldDefDto>),
    SetSortState { field: SharedString, descending: bool },
    SetTotalProcessesCount(usize),
    SetEmptyStateVisible(bool),
    SetEmptyStateTitle(SharedString),
    SetEmptyStateMessage(SharedString),
    SetIsGrouped(bool),
    SetSelectedPid(i32),
    SetSelectedName(SharedString),
}

#[port]
pub trait UiProcessesPort: Debug + 'static {
    fn send(&self, msg: UiProcessesPortMsg);

    // TODO: the only port method that isn't `send(msg)` - it exists purely
    // as a synchronous UI->domain query and is the sole reason the
    // `PortStubMeta::extra_methods` machinery (forsl_core::contracts,
    // forsl-codegen's parse_port_extra_methods/generate_port_extra_method)
    // exists at all. Rework selected-pid tracking to flow through a
    // message/signal instead, then delete this method and that whole
    // side-channel.
    fn get_selected_pid(&self) -> i32;
}
