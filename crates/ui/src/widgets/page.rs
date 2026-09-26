use windows_reactor::{
    Border, Button, ButtonStyle, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength,
    HorizontalAlignment, LayoutControl, Orientation, ProgressRing, StackPanel, Thickness,
    VerticalAlignment, View,
};

use guinea::winui::MarkExt;
use guinea::Mark;

use crate::theme::{space, Palette};
use crate::widgets::card::card;
use crate::widgets::separator;
use crate::widgets::text::{body_large, text};

pub fn page_frame(
    header: impl Into<View>,
    body: impl Into<View>,
    status: impl Into<String>,
    palette: Palette,
) -> View {
    let header_card = card()
        .grid_row(0)
        .margin(Thickness::new(0.0, 0.0, 0.0, space::Control))
        .padding(Thickness::xy(space::Header, space::Control))
        .content(header);

    let content_card = card().grid_row(1).content(
        Grid::new()
            .rows([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
            .children((
                Border::new().grid_row(0).content(body),
                separator(palette).grid_row(1),
                Border::new()
                    .grid_row(2)
                    .padding(Thickness::xy(space::Header, space::Control))
                    .content(text(status).foreground(palette.secondary_text)),
            )),
    );

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .margin(Thickness::uniform(space::Control))
        .children((header_card, content_card))
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

pub fn command_button(
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    labelled_button(ButtonStyle::Subtle, mark, label, icon, enabled, on_click)
}

pub fn action_button(
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    on_click: impl Fn() + 'static,
) -> View {
    labelled_button(ButtonStyle::Default, mark, label, icon, true, on_click)
}

fn labelled_button(
    style: ButtonStyle,
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    let button = Button::new()
        .mark(mark)
        .style(style)
        .is_enabled(enabled)
        .on_click(on_click);
    match icon {
        Some(icon) => button.content(
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((icon, text(label))),
        ),
        None => button.content(text(label)),
    }
}
