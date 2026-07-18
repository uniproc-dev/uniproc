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

pub trait UiProcessesPort: Debug + 'static {
    fn send(&self, msg: UiProcessesPortMsg);
    fn get_selected_pid(&self) -> i32;
}
