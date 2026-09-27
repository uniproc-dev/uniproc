use windows_reactor::{
    ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, LayoutControl, StackPanel,
    Thickness, VerticalAlignment, View,
};

use crate::theme::{space, Palette};
use crate::widgets::card::card;
use crate::widgets::text::{caption, text};

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
        .padding(Thickness::xy(space::Card, space::Header))
        .content(
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .column_spacing(space::Card)
                .children((
                    Grid::new()
                        .grid_column(0)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((icon,)),
                    Grid::new().grid_column(1).children((words,)),
                    Grid::new()
                        .grid_column(2)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((control,)),
                )),
        )
        .into()
}
