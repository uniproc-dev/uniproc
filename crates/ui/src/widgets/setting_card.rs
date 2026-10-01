use windows_reactor::{
    ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, LayoutControl, StackPanel,
    ThemeBrush, Thickness, VerticalAlignment, View,
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

pub fn setting_card(setting: SettingCard, palette: Palette) -> View {
    let SettingCard {
        icon,
        title,
        description,
        control,
    } = setting;

    let words = StackPanel::new()
        .vertical_alignment(VerticalAlignment::Center)
        .children((
            text(title),
            caption(description).foreground(palette.secondary_text),
        ));
    let icon = match icon {
        Some(icon) => Grid::new()
            .grid_column(0)
            .margin(setting::icon_margin())
            .vertical_alignment(VerticalAlignment::Center)
            .children((icon,))
            .into(),
        None => View::empty(),
    };

    card()
        .corner_radius(radius::Control)
        .border_thickness(SettingCardSize::Border)
        .border_brush(ThemeBrush::CardStroke)
        .min_height(setting::CardMinHeight)
        .padding(Thickness::uniform(space::Card))
        .content(
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .children((
                    icon,
                    Grid::new()
                        .grid_column(1)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((words,)),
                    Grid::new()
                        .grid_column(2)
                        .min_width(SettingCardSize::ContentMinWidth)
                        .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((control,)),
                )),
        )
        .into()
}
