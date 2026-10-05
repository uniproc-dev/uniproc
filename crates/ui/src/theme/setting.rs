#![expect(non_upper_case_globals)]

use windows_reactor::Thickness;

pub const Icon: f64 = 20.0;
pub const CardMinHeight: f64 = 68.0;
pub const CardSpacing: f64 = 4.0;
pub const Choice: f64 = 180.0;
pub const ExpanderHeaderInset: f64 = 16.0;
pub const ExpanderStart: f64 = 16.0;
pub const ExpanderEnd: f64 = 44.0;
pub const ExpanderRowMinHeight: f64 = 52.0;
pub const ExpanderRowInset: f64 = 8.0;
pub const CaptionInset: f64 = 1.0;

pub fn icon_margin() -> Thickness {
    Thickness::new(2.0, 0.0, 20.0, 0.0)
}

pub fn icon_column() -> f64 {
    let margin = icon_margin();
    margin.left + Icon + margin.right
}
