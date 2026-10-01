use windows_reactor::{
    Border, Button, ButtonStyle, Callback, ChildrenControl, Color, ContentControl, Grid,
    GridChildExt, GridLength, HorizontalAlignment, IntoViews, LayoutControl, Orientation, PointerEventInfo,
    ProgressRing, ResourceOverrides, ScrollViewer, StackPanel, Thickness, VerticalAlignment, View,
};

use guinea::winui::MarkExt;
use guinea::Mark;

use crate::theme::{opacity, space, Palette};
use crate::widgets::card::card;
use crate::widgets::separator;
use crate::widgets::text::{body_large, body_strong, text};

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

struct SettingsColumn;

#[expect(non_upper_case_globals)]
impl SettingsColumn {
    const MaxWidth: f64 = 1064.0;
    const CardSpacing: f64 = 4.0;

    fn section_header() -> Thickness {
        Thickness::new(1.0, 30.0, 0.0, 6.0)
    }
}

pub fn settings_section(title: impl Into<String>, cards: impl IntoViews) -> View {
    let header = body_strong(title).margin(SettingsColumn::section_header());
    let cards = StackPanel::new().spacing(SettingsColumn::CardSpacing).children(cards);
    StackPanel::new().children((header, cards)).into()
}

pub fn settings_column(children: impl IntoViews) -> View {
    let column = StackPanel::new()
        .max_width(SettingsColumn::MaxWidth)
        .margin(Thickness::new(space::Page, space::Section, space::Page, space::Page))
        .children(children);
    ScrollViewer::new().content(Grid::new().children((column,))).into()
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

struct IconButton;

#[expect(non_upper_case_globals)]
impl IconButton {
    const Padding: f64 = 6.0;
}

pub fn icon_button(mark: impl Mark, icon: View, on_click: impl Fn() + 'static) -> View {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .resource_overrides(ResourceOverrides::new().set("ButtonPadding", Thickness::uniform(IconButton::Padding)))
        .on_click(on_click)
        .content(icon)
        .into()
}

pub fn action_button(
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    labelled_button(ButtonStyle::Default, mark, label, icon, enabled, on_click)
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
                .children((
                    Border::new()
                        .opacity(if enabled { 1.0 } else { opacity::Disabled })
                        .content(icon),
                    text(label),
                )),
        ),
        None => button.content(text(label)),
    }
}
