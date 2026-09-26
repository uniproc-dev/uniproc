use windows_reactor::{LayoutControl, Rectangle};

use crate::theme::Palette;

pub fn separator(palette: Palette) -> Rectangle {
    Rectangle::new().fill(palette.divider_stroke).height(1.0)
}
