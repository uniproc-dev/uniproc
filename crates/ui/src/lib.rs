#[allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::all
)]
mod bindings;
pub mod format;
pub mod l10n;
mod layouts;
pub mod pages;
pub mod theme;
pub mod widgets;

pub use layouts::{
    ServiceTrouble, ShellNav, ShellProps, SidebarChartsProps, SidebarMark, SplashMark, SplashProps,
    shell_view, sidebar_charts, splash_view,
};
