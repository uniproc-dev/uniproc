#![expect(non_upper_case_globals)]

use windows_reactor::Thickness;

pub const Icon: f64 = 20.0;
pub const CardMinHeight: f64 = 68.0;
pub const CardSpacing: f64 = 4.0;
pub const Choice: f64 = 180.0;
pub const ExpanderHeaderInset: f64 = 12.0;
pub const CaptionInset: f64 = 1.0;

pub fn icon_margin() -> Thickness {
    Thickness::new(2.0, 0.0, 20.0, 0.0)
}
