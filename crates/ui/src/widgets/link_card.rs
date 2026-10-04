use guicons::icon;
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Button, Grid, GridLength, HorizontalAlignment, ResourceOverrides, Thickness, VerticalAlignment, View,
};

use crate::theme::{setting, space, Palette};
use crate::widgets::setting_card::card_words;

struct LinkCardSize;

#[expect(non_upper_case_globals)]
impl LinkCardSize {
    const Glyph: f64 = 16.0;

    fn content_margin() -> Thickness {
        let button = Thickness::new(11.0, 5.0, 11.0, 6.0);
        Thickness::new(
            space::Card - button.left,
            space::Card - button.top,
            space::Card - button.right,
            space::Card - button.bottom,
        )
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
    glyph.size(LinkCardSize::Glyph).build_element()
}

fn card_look(palette: Palette) -> ResourceOverrides {
    ResourceOverrides::new()
        .set("ButtonBackground", palette.card_fill)
        .set("ButtonBackgroundPointerOver", palette.card_fill_hovered)
        .set("ButtonBackgroundPressed", palette.card_fill_pressed)
        .set("ButtonBorderBrush", palette.card_stroke)
        .set("ButtonBorderBrushPointerOver", palette.card_stroke)
        .set("ButtonBorderBrushPressed", palette.card_stroke)
}

pub fn link_card(mark: impl Mark, card: LinkCard, palette: Palette, on_click: impl Fn() + 'static) -> View {
    let LinkCard {
        icon,
        title,
        description,
        trailing,
        accessory,
    } = card;

    let words = card_words(title, description, palette);
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
    if let Some(accessory) = accessory {
        parts.push(
            Grid::new()
                .grid_column(2)
                .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
                .vertical_alignment(VerticalAlignment::Center)
                .children((accessory,))
                .into(),
        );
    }
    parts.push(
        Grid::new()
            .grid_column(3)
            .margin(Thickness::new(space::Card, 0.0, 0.0, 0.0))
            .vertical_alignment(VerticalAlignment::Center)
            .children((trailing_glyph(trailing),))
            .into(),
    );

    Button::new()
        .mark(mark)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .horizontal_content_alignment(HorizontalAlignment::Stretch)
        .min_height(setting::CardMinHeight)
        .resource_overrides(card_look(palette))
        .on_click(on_click)
        .content(
            Grid::new()
                .margin(LinkCardSize::content_margin())
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
                .children(parts),
        )
        .into()
}
