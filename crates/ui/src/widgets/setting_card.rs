use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Color, ComboBox, Expander, Grid, GridLength, HorizontalAlignment, ResourceOverrides, StackPanel, ThemeBrush,
    Thickness, VerticalAlignment, View,
};

use crate::theme::{radius, setting, space, Palette};
use crate::widgets::card::card;
use crate::widgets::text::{caption, text};

struct SettingCardSize;

#[expect(non_upper_case_globals)]
impl SettingCardSize {
    const Border: f64 = 1.0;
    const ContentMinWidth: f64 = 120.0;
}

pub struct SettingCard {
    pub icon: Option<View>,
    pub title: String,
    pub description: String,
    pub control: View,
}

pub fn card_words(title: impl Into<String>, description: Option<String>, palette: Palette) -> View {
    let mut words: Vec<View> = vec![text(title).into()];
    if let Some(description) = description {
        words.push(caption(description).foreground(palette.secondary_text).into());
    }
    StackPanel::new()
        .vertical_alignment(VerticalAlignment::Center)
        .children(words)
        .into()
}

pub fn setting_expander() -> Expander {
    Expander::new().horizontal_alignment(HorizontalAlignment::Stretch).resource_overrides(
        ResourceOverrides::new()
            .set("ExpanderChevronPointerOverBackground", Color::transparent())
            .set("ExpanderChevronPressedBackground", Color::transparent()),
    )
}

pub fn choice<T: Copy + PartialEq + 'static>(
    mark: impl Mark,
    all: &'static [T],
    current: T,
    label: impl Fn(T) -> String,
    pick: impl Fn(T) + 'static,
) -> View {
    ComboBox::new()
        .mark(mark)
        .width(setting::Choice)
        .items_source(all.iter().map(|item| label(*item)))
        .selected_index(all.iter().position(|item| *item == current))
        .on_selection_changed(move |index: Option<usize>| {
            if let Some(item) = index.and_then(|index| all.get(index)) {
                pick(*item);
            }
        })
        .into()
}

pub fn setting_card(setting: SettingCard, palette: Palette) -> View {
    let SettingCard {
        icon,
        title,
        description,
        control,
    } = setting;

    let words = card_words(title, Some(description), palette);
    let mut parts: Vec<View> = Vec::new();
    if let Some(icon) = icon {
        parts.push(
            Grid::new()
                .grid_column(0)
                .margin(setting::icon_margin())
                .vertical_alignment(VerticalAlignment::Center)
                .children((icon,))
                .into(),
        );
    }
    parts.push(
        Grid::new()
            .grid_column(1)
            .vertical_alignment(VerticalAlignment::Center)
            .children((words,))
            .into(),
    );
    parts.push(
        Grid::new()
            .grid_column(2)
            .min_width(SettingCardSize::ContentMinWidth)
            .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
            .vertical_alignment(VerticalAlignment::Center)
            .children((control,))
            .into(),
    );

    card()
        .corner_radius(radius::Control)
        .border_thickness(SettingCardSize::Border)
        .border_brush(ThemeBrush::CardStroke)
        .min_height(setting::CardMinHeight)
        .padding(Thickness::uniform(space::Card))
        .content(
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .children(parts),
        )
        .into()
}
