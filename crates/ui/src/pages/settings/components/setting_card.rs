use windows_reactor::{
    ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, LayoutControl, StackPanel,
    ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::theme::{radius, space, Palette};
use crate::widgets::card::card;
use crate::widgets::text::{caption, text};

pub(crate) struct SettingCardSize;

#[expect(non_upper_case_globals)]
impl SettingCardSize {
    pub(crate) const Icon: f64 = 20.0;
    const MinHeight: f64 = 68.0;
    const Border: f64 = 1.0;
    const ContentMinWidth: f64 = 120.0;

    fn icon_margin() -> Thickness {
        Thickness::new(2.0, 0.0, 20.0, 0.0)
    }
}

pub(crate) struct SettingCard {
    pub(crate) icon: View,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) control: View,
}

pub(crate) fn setting_card(setting: SettingCard, palette: Palette) -> View {
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

    card()
        .corner_radius(radius::Control)
        .border_thickness(SettingCardSize::Border)
        .border_brush(ThemeBrush::CardStroke)
        .min_height(SettingCardSize::MinHeight)
        .padding(Thickness::uniform(space::Card))
        .content(
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .children((
                    Grid::new()
                        .grid_column(0)
                        .margin(SettingCardSize::icon_margin())
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((icon,)),
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
