use guicons::icon;
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Button, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, HorizontalAlignment, LayoutControl,
    ResourceOverrides, StackPanel, Thickness, VerticalAlignment, View,
};

use crate::theme::{space, Palette};
use crate::widgets::text::{caption, text};

struct LinkCardSize;

#[expect(non_upper_case_globals)]
impl LinkCardSize {
    const MinHeight: f64 = 68.0;
    const Glyph: f64 = 16.0;

    fn icon_margin() -> Thickness {
        Thickness::new(2.0, 0.0, 20.0, 0.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trailing {
    External,
    Download,
    Chevron,
}

pub struct LinkCard {
    pub icon: Option<View>,
    pub title: String,
    pub description: Option<String>,
    pub trailing: Trailing,
    pub accessory: Option<View>,
}

fn trailing_glyph(trailing: Trailing) -> View {
    let glyph = match trailing {
        Trailing::External => icon!(open_external),
        Trailing::Download => icon!(download_regular),
        Trailing::Chevron => icon!(chevron_right_regular),
    };
    glyph.size(LinkCardSize::Glyph).build()
}

fn card_look(palette: Palette) -> ResourceOverrides {
    ResourceOverrides::new()
        .set("ButtonBackground", palette.card_fill)
        .set("ButtonBackgroundPointerOver", palette.card_fill_hovered)
        .set("ButtonBackgroundPressed", palette.card_fill_pressed)
        .set("ButtonBorderBrush", palette.card_stroke)
        .set("ButtonBorderBrushPointerOver", palette.card_stroke)
        .set("ButtonBorderBrushPressed", palette.card_stroke)
        .set("ButtonPadding", Thickness::uniform(space::Card))
}

pub fn link_card(mark: impl Mark, card: LinkCard, palette: Palette, on_click: impl Fn() + 'static) -> View {
    let LinkCard {
        icon,
        title,
        description,
        trailing,
        accessory,
    } = card;

    let words = StackPanel::new().vertical_alignment(VerticalAlignment::Center).children((
        text(title),
        match description {
            Some(description) => caption(description).foreground(palette.secondary_text).into(),
            None => View::empty(),
        },
    ));
    let icon = match icon {
        Some(icon) => Grid::new()
            .grid_column(0)
            .margin(LinkCardSize::icon_margin())
            .vertical_alignment(VerticalAlignment::Center)
            .children((icon,))
            .into(),
        None => View::empty(),
    };
    let accessory = match accessory {
        Some(accessory) => Grid::new()
            .grid_column(2)
            .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
            .vertical_alignment(VerticalAlignment::Center)
            .children((accessory,))
            .into(),
        None => View::empty(),
    };

    Button::new()
        .mark(mark)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .horizontal_content_alignment(HorizontalAlignment::Stretch)
        .min_height(LinkCardSize::MinHeight)
        .resource_overrides(card_look(palette))
        .on_click(on_click)
        .content(
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
                .children((
                    icon,
                    Grid::new()
                        .grid_column(1)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((words,)),
                    accessory,
                    Grid::new()
                        .grid_column(3)
                        .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((trailing_glyph(trailing),)),
                )),
        )
        .into()
}
