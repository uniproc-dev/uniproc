use windows_reactor::{
    Border, Grid, GridLength, HorizontalAlignment, ProgressRing, Thickness, VerticalAlignment, View,
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
}

pub fn status_text(status: impl Into<String>, palette: Palette) -> View {
    text(status).foreground(palette.secondary_text).into()
}

pub fn page_frame(header: impl Into<View>, body: impl Into<View>, status: impl Into<View>, palette: Palette) -> View {
    let header_card = card()
        .mark(PageMark::Header)
        .grid_row(0)
        .margin(Thickness::new(0.0, 0.0, 0.0, space::Control))
        .padding(Thickness::xy(space::Header, space::Control))
        .content(header);

    let status_bar = Border::new()
        .mark(PageMark::Status)
        .grid_row(2)
        .padding(Thickness::xy(space::Header, space::Control))
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

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .margin(Thickness::new(0.0, space::Control, space::Control, space::Control))
        .children((header_card, content_card))
        .into()
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
