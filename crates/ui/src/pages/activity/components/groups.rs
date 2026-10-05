use app_contracts::features::activity::Group;
use windows_reactor::{Border, Color, CornerRadius, Thickness, VerticalAlignment, View};

use crate::l10n::L10n;
use crate::theme::space;

struct Swatch;

#[expect(non_upper_case_globals)]
impl Swatch {
    const Size: f64 = 10.0;
}

pub fn group_label(group: &Group, l10n: &L10n) -> String {
    if group.is_built_in() {
        l10n.activity_group_windows_background()
    } else {
        group.name.clone()
    }
}

pub fn swatch(color: Color, filled: bool) -> View {
    let dot = Border::new()
        .width(Swatch::Size)
        .height(Swatch::Size)
        .corner_radius(CornerRadius::uniform(Swatch::Size / 2.0))
        .vertical_alignment(VerticalAlignment::Center)
        .border_brush(color)
        .border_thickness(Thickness::uniform(space::Hairline));
    if filled { dot.background(color) } else { dot }.into()
}
