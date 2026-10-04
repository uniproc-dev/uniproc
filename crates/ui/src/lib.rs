#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::all)]
mod bindings;
mod layouts;
pub mod format;
pub mod l10n;
pub mod pages;
pub mod theme;
pub mod widgets;

pub use layouts::{
    shell_view, sidebar_charts, splash_view, ServiceTrouble, ShellNav, ShellProps, SidebarChartsProps, SidebarMark,
    SplashMark, SplashProps,
};
