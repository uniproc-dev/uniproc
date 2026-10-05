use guinea::Mark;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, Color, CornerRadius, Grid, GridLength,
    HorizontalAlignment, Orientation, PointerEventInfo, StackPanel, Thickness, VerticalAlignment,
    View,
};

use crate::theme::{Palette, radius, space};
use crate::widgets::separator;
use crate::widgets::text::{caption, text};

struct Menu;

#[expect(non_upper_case_globals)]
impl Menu {
    const Width: f64 = 248.0;
    const ShadowDrop: f64 = 2.0;
}

pub struct MenuEntry<M, C> {
    pub mark: M,
    pub icon: View,
    pub label: String,
    pub enabled: bool,
    pub command: C,
}

pub enum MenuLine<M, C> {
    Entry(MenuEntry<M, C>),
    Caption(String),
    Separator,
}

impl<M, C> MenuLine<M, C> {
    pub fn entry(mark: M, icon: View, label: String, command: C) -> Self {
        Self::Entry(MenuEntry {
            mark,
            icon,
            label,
            enabled: true,
            command,
        })
    }

    pub fn enabled_if(self, enabled: bool) -> Self {
        match self {
            Self::Entry(entry) => Self::Entry(MenuEntry { enabled, ..entry }),
            line => line,
        }
    }
}

pub struct PopupMenu<M, C> {
    pub x: f64,
    pub y: f64,
    pub lines: Vec<MenuLine<M, C>>,
    pub card: M,
    pub backdrop: M,
    pub palette: Palette,
    pub on_command: Callback<C>,
    pub on_dismiss: Callback<()>,
}

fn line_view<M: Mark, C: Clone + 'static>(
    line: MenuLine<M, C>,
    on_command: &Callback<C>,
    palette: Palette,
) -> View {
    match line {
        MenuLine::Separator => separator(palette)
            .margin(Thickness::xy(0.0, space::Compact))
            .into(),
        MenuLine::Caption(words) => caption(words)
            .foreground(palette.secondary_text)
            .margin(Thickness::new(
                space::Cell,
                space::Compact,
                space::Cell,
                space::Hairline,
            ))
            .into(),
        MenuLine::Entry(entry) => {
            let on_command = on_command.clone();
            let command = entry.command;
            Button::new()
                .mark(entry.mark)
                .style(ButtonStyle::Subtle)
                .is_enabled(entry.enabled)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .horizontal_content_alignment(HorizontalAlignment::Left)
                .on_click(move || on_command.call(command.clone()))
                .content(
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(space::Header)
                        .children((entry.icon, text(entry.label))),
                )
                .into()
        }
    }
}

pub fn popup_menu<M: Mark, C: Clone + 'static>(menu: PopupMenu<M, C>) -> View {
    let PopupMenu {
        x,
        y,
        lines,
        card,
        backdrop,
        palette,
        on_command,
        on_dismiss,
    } = menu;
    let items: Vec<View> = lines
        .into_iter()
        .map(|line| line_view(line, &on_command, palette))
        .collect();

    let card = Border::new()
        .mark(card)
        .grid_row(1)
        .grid_column(1)
        .width(Menu::Width)
        .background(palette.menu_fill)
        .border_brush(palette.menu_stroke)
        .border_thickness(Thickness::uniform(space::Hairline))
        .corner_radius(radius::Overlay)
        .padding(Thickness::uniform(space::Compact))
        .content(StackPanel::new().children(items));

    let shadow = Border::new()
        .grid_row(1)
        .grid_column(1)
        .background(palette.menu_shadow)
        .corner_radius(CornerRadius::uniform(radius::Overlay + space::Hairline))
        .margin(Thickness::new(
            -space::Hairline,
            Menu::ShadowDrop,
            -space::Hairline,
            -Menu::ShadowDrop,
        ));

    let placed = Grid::new()
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Top)
        .rows([GridLength::Star(1.0), GridLength::Auto])
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .children((
            Border::new().grid_row(0).grid_column(0).width(x).height(y),
            shadow,
            card,
        ));

    let backdrop = Border::new()
        .mark(backdrop)
        .background(Color::transparent())
        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
            on_dismiss.call(());
        }));

    Grid::new().children((backdrop, placed)).into()
}
