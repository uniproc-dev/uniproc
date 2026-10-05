use windows_reactor::{Grid, IntoViews, ScrollViewer, StackPanel, Thickness, View};

use crate::theme::{setting, space};
use crate::widgets::text::body_strong;

struct SettingsColumn;

#[expect(non_upper_case_globals)]
impl SettingsColumn {
    const MaxWidth: f64 = 1064.0;

    fn section_header() -> Thickness {
        Thickness::new(setting::CaptionInset, 30.0, 0.0, 6.0)
    }
}

pub fn settings_section(title: impl Into<String>, cards: impl IntoViews) -> View {
    let header = body_strong(title).margin(SettingsColumn::section_header());
    let cards = StackPanel::new()
        .spacing(setting::CardSpacing)
        .children(cards);
    StackPanel::new().children((header, cards)).into()
}

pub fn settings_column(children: impl IntoViews) -> View {
    let column = StackPanel::new()
        .max_width(SettingsColumn::MaxWidth)
        .margin(Thickness::new(
            space::Page,
            space::Section,
            space::Page,
            space::Page,
        ))
        .children(children);
    ScrollViewer::new()
        .content(Grid::new().children((column,)))
        .into()
}
