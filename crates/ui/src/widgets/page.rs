use windows_reactor::{
    Border, Callback, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength,
    HorizontalAlignment, LayoutControl, PointerEventInfo, ProgressRing, Thickness, VerticalAlignment, View,
};

use guinea::winui::MarkExt;

use crate::theme::{space, Palette};
use crate::widgets::card::card;
use crate::widgets::separator;
use crate::widgets::text::{body_large, text};

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageMark {
    Header,
    Status,
    Blank,
}

pub fn status_text(status: impl Into<String>, palette: Palette) -> View {
    text(status).foreground(palette.secondary_text).into()
}

fn on_blank(on_blank: &Option<Callback<()>>) -> Option<Callback<PointerEventInfo>> {
    on_blank.clone().map(|on_blank| {
        Callback::new(move |_: PointerEventInfo| {
            let _ = on_blank.call(());
        })
    })
}

pub fn page_frame(
    header: impl Into<View>,
    body: impl Into<View>,
    status: impl Into<View>,
    palette: Palette,
    blank: Option<Callback<()>>,
) -> View {
    let header_card = card()
        .mark(PageMark::Header)
        .grid_row(0)
        .margin(Thickness::new(0.0, 0.0, 0.0, space::Control))
        .padding(Thickness::xy(space::Header, space::Control));
    let header_card = match on_blank(&blank) {
        Some(released) => header_card.on_pointer_released(released),
        None => header_card,
    }
    .content(header);

    let status_bar = Border::new()
        .mark(PageMark::Status)
        .grid_row(2)
        .padding(Thickness::xy(space::Header, space::Control));
    let status_bar = match on_blank(&blank) {
        Some(released) => status_bar.background(Color::transparent()).on_pointer_released(released),
        None => status_bar,
    }
    .content(status);

    let content_card = card().grid_row(1).content(
        Grid::new()
            .rows([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
            .children((
                Border::new().grid_row(0).content(body),
                separator(palette).grid_row(1),
                status_bar,
            )),
    );

    let cards = Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .margin(Thickness::new(0.0, space::Control, space::Control, space::Control))
        .children((header_card, content_card));

    let under = match on_blank(&blank) {
        Some(released) => Border::new()
            .mark(PageMark::Blank)
            .background(Color::transparent())
            .on_pointer_released(released)
            .into(),
        None => View::empty(),
    };

    Grid::new().children((under, cards)).into()
}

pub fn page_title(title: impl Into<String>) -> View {
    body_large(title)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

pub fn loading() -> View {
    ProgressRing::new()
        .is_indeterminate(true)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}
