use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    keyed, Border, Color, ComboBox, Expander, Grid, GridLength, HorizontalAlignment, KeyedView, Orientation,
    ResourceOverrides, StackPanel, ThemeBrush, Thickness, ToggleSwitch, VerticalAlignment, View,
};

use crate::theme::{radius, setting, space, Palette};
use crate::widgets::card::card;
use crate::widgets::text::{caption, text};

struct SettingCardSize;

#[expect(non_upper_case_globals)]
impl SettingCardSize {
    const Border: f64 = 1.0;
    const ContentMinWidth: f64 = 120.0;
    const WinuiExpanderContentPadding: f64 = 16.0;
    const WinuiSwitchHeight: f64 = 40.0;
    const WinuiSwitchEmptyContent: f64 = 12.0;
    const SwitchHeight: f64 = 36.0;
}

pub fn setting_switch(
    mark: impl Mark,
    on: bool,
    enabled: bool,
    state: String,
    palette: Palette,
    on_toggled: impl Fn(bool) + 'static,
) -> View {
    let state = text(state).vertical_alignment(VerticalAlignment::Center);
    let state = if enabled { state } else { state.foreground(palette.disabled_text) };
    let trim = (SettingCardSize::WinuiSwitchHeight - SettingCardSize::SwitchHeight) / 2.0;
    let switch = ToggleSwitch::new()
        .mark(mark)
        .is_on(on)
        .is_enabled(enabled)
        .on_content(text(""))
        .off_content(text(""))
        .min_width(0.0)
        .margin(Thickness::new(0.0, -trim, -SettingCardSize::WinuiSwitchEmptyContent, -trim))
        .on_toggled(on_toggled);
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Header)
        .children((state, switch))
        .into()
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
    Expander::new()
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .horizontal_content_alignment(HorizontalAlignment::Stretch)
        .resource_overrides(
            ResourceOverrides::new()
                .set("ExpanderChevronPointerOverBackground", Color::transparent())
                .set("ExpanderChevronPressedBackground", Color::transparent()),
        )
}

fn bands(rows: Vec<(String, View)>, start: f64, end: f64) -> StackPanel {
    let bands: Vec<KeyedView> = rows
        .into_iter()
        .enumerate()
        .map(|(at, (key, view))| {
            let band = Border::new()
                .min_height(setting::ExpanderRowMinHeight)
                .padding(Thickness::new(start, setting::ExpanderRowInset, end, setting::ExpanderRowInset))
                .border_thickness(Thickness::new(0.0, if at == 0 { 0.0 } else { space::Hairline }, 0.0, 0.0))
                .border_brush(ThemeBrush::CardStroke)
                .content(Grid::new().vertical_alignment(VerticalAlignment::Center).children((view,)));
            keyed(key, band)
        })
        .collect();
    StackPanel::new().keyed_children(bands)
}

pub fn expander_rows(indent: f64, rows: Vec<(String, View)>) -> StackPanel {
    bands(rows, setting::ExpanderStart + indent, setting::ExpanderEnd)
        .margin(Thickness::uniform(-SettingCardSize::WinuiExpanderContentPadding))
}

pub fn card_rows(rows: Vec<(String, View)>) -> View {
    card()
        .corner_radius(radius::Control)
        .border_thickness(SettingCardSize::Border)
        .border_brush(ThemeBrush::CardStroke)
        .content(bands(rows, space::Card, space::Card))
        .into()
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
