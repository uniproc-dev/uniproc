use windows_reactor::{Border, ThemeBrush};

use crate::theme::radius;

pub fn card() -> Border {
    Border::new()
        .background(ThemeBrush::CardBackground)
        .corner_radius(radius::Card)
}
