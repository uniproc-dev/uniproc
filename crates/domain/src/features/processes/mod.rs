mod actor;
mod rates;
pub mod shell;
pub mod windows_scan;
mod install;
pub mod settings;
mod wsl_rows;

pub use actor::rows_from_report;
pub use install::{ProcessesDeps, ProcessesFeature};
