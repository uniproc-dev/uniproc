use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{ActivityRow, Came, Launcher, Series, Went};
use app_contracts::features::agents::ProcessInstance;
use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    keyed, Border, Callback, Color, Component, ComponentContext, CornerRadius, Grid, GridLength, HorizontalAlignment,
    KeyedView, Orientation, PointerEventInfo, StackPanel, TextTrimming, TextWrapping, Thickness, VerticalAlignment, View,
    ViewContext,
};

use super::super::marks::ActivityMark;
use super::facts::{facts, went_facts};
use super::lasted::lasted;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, radius, size, space, Palette};
use crate::widgets::text::{caption, text};

struct Line;

#[expect(non_upper_case_globals)]
impl Line {
    const Icon: f64 = 20.0;
    const Indent: f64 = 80.0;
    const Stripe: f64 = 3.0;
}

pub struct Rows {
    pub rows: Rc<[ActivityRow]>,
    pub earlier: usize,
    pub expanded: Rc<HashSet<ProcessInstance>>,
    pub selected: Option<ProcessInstance>,
    pub l10n: L10n,
    pub palette: Palette,
    pub on_press: Callback<ProcessInstance>,
}

#[derive(Clone, PartialEq)]
struct Item {
    row: ActivityRow,
    open: bool,
    selected: bool,
    l10n: L10n,
    palette: Palette,
    on_press: Callback<ProcessInstance>,
}

enum Pointer {
    Entered,
    Exited,
}

struct ItemView(bool);

impl Component for ItemView {
    type Input = Item;
    type Message = Pointer;

    fn create(_item: &Item, _cx: &ComponentContext<Self>) -> Self {
        Self(false)
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        self.0 = matches!(message, Pointer::Entered);
    }

    fn view(&self, item: &Item, cx: &mut ViewContext<Self>) -> View {
        let plate = Border::new()
            .margin(Thickness::xy(space::Compact, space::Hairline))
            .corner_radius(radius::Control);
        let plate = match (item.selected, self.0) {
            (true, _) => plate.mark(ActivityMark::Selected).background(item.palette.row_selected),
            (false, true) => plate.background(item.palette.row_hovered),
            (false, false) => plate,
        };
        let mut layers: Vec<View> = vec![plate.into()];
        if let Some(hue) = item.row.hue() {
            layers.push(
                Border::new()
                    .mark(ActivityMark::Stripe)
                    .width(Line::Stripe)
                    .horizontal_alignment(HorizontalAlignment::Left)
                    .margin(Thickness::xy(space::Compact, space::Control))
                    .corner_radius(CornerRadius::uniform(Line::Stripe / 2.0))
                    .background(item.palette.hue(hue))
                    .into(),
            );
        }
        layers.push(row_view(item));
        Border::new()
            .background(Color::transparent())
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Pointer::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Exited))
            .content(Grid::new().children(layers))
            .into()
    }
}

fn key(instance: ProcessInstance) -> String {
    format!("{}:{}", instance.pid, instance.sequence)
}

pub fn rows(list: Rows) -> View {
    let mut children: Vec<KeyedView> = list
        .rows
        .iter()
        .map(|row| {
            let item = Item {
                row: row.clone(),
                open: list.expanded.contains(&row.key()),
                selected: list.selected == Some(row.key()),
                l10n: list.l10n.clone(),
                palette: list.palette,
                on_press: list.on_press.clone(),
            };
            keyed(key(row.key()), View::component::<ItemView>(item))
        })
        .collect();
    if list.earlier > 0 {
        children.push(keyed(
            "earlier",
            caption(list.l10n.activity_earlier(list.earlier as i64))
                .foreground(list.palette.secondary_text)
                .margin(Thickness::uniform(space::Cell)),
        ));
    }
    StackPanel::new().mark(ActivityMark::Rows).keyed_children(children).into()
}

fn centered(content: impl Into<String>) -> View {
    text(content).vertical_alignment(VerticalAlignment::Center).into()
}

fn secondary(content: impl Into<String>, palette: Palette) -> View {
    caption(content)
        .foreground(palette.secondary_text)
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

fn spaced(parts: Vec<KeyedView>) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Control)
        .vertical_alignment(VerticalAlignment::Center)
        .keyed_children(parts)
        .into()
}

fn line(at: String, glyph: View, main: View, trailing: View, palette: Palette) -> View {
    Grid::new()
        .columns([GridLength::Auto, GridLength::Pixel(Line::Icon), GridLength::Star(1.0), GridLength::Auto])
        .min_height(size::ProcessRow)
        .children((
            caption(at)
                .grid_column(0)
                .foreground(palette.secondary_text)
                .margin(Thickness::new(0.0, 0.0, space::Control, 0.0))
                .vertical_alignment(VerticalAlignment::Center),
            Border::new()
                .grid_column(1)
                .vertical_alignment(VerticalAlignment::Center)
                .content(glyph),
            Border::new()
                .grid_column(2)
                .vertical_alignment(VerticalAlignment::Center)
                .margin(Thickness::xy(space::Control, 0.0))
                .content(main),
            Border::new()
                .grid_column(3)
                .vertical_alignment(VerticalAlignment::Center)
                .content(trailing),
        ))
        .into()
}

fn pressed(content: impl Into<View>, key: ProcessInstance, on_press: &Callback<ProcessInstance>) -> View {
    let on_press = on_press.clone();
    Border::new()
        .mark(ActivityMark::Row)
        .background(Color::transparent())
        .padding(Thickness::xy(space::Cell, 0.0))
        .on_pointer_released(move |_: PointerEventInfo| on_press.call(key))
        .content(content)
        .into()
}

fn joined(names: &[std::sync::Arc<str>], l10n: &L10n) -> String {
    names
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
        .join(&l10n.activity_services_separator())
}

pub fn launcher_text(came: &Came, l10n: &L10n) -> Option<String> {
    match &came.launcher {
        Launcher::Task(task) => Some(l10n.activity_from_task(task.name.to_string())),
        Launcher::Services(names) => Some(l10n.activity_from_services(joined(names, l10n))),
        Launcher::Process(name) => Some(l10n.activity_from(name.to_string())),
        Launcher::Unknown => None,
    }
}

fn came_main(came: &Came, l10n: &L10n, palette: Palette) -> View {
    let mut parts = vec![keyed("name", centered(came.name.to_string()))];
    if let Some(from) = launcher_text(came, l10n) {
        parts.push(keyed("from", secondary(from, palette)));
    }
    if came.first_seen {
        parts.push(keyed(
            "first-seen",
            caption(l10n.activity_first_seen())
                .foreground(accent_color())
                .vertical_alignment(VerticalAlignment::Center),
        ));
    }
    spaced(parts)
}

fn came_view(item: &Item, came: &Came) -> View {
    let Item { l10n, palette, on_press, .. } = item;
    let trailing = match &came.exit {
        Some(exit) => lasted(l10n, exit.lived),
        None => l10n.activity_still_running(),
    };
    let head = line(
        format::clock(came.at),
        icon!(came).size(size::Icon).build_element(),
        came_main(came, l10n, *palette),
        secondary(trailing, *palette),
        *palette,
    );
    let body = if item.open {
        StackPanel::new().children((head, facts(came, l10n, *palette, Line::Indent))).into()
    } else {
        head
    };
    pressed(body, came.key, on_press)
}

fn went_view(item: &Item, went: &Went) -> View {
    let Item { l10n, palette, on_press, .. } = item;
    let parts = match &went.name {
        Some(name) => vec![keyed("name", centered(name.to_string()))],
        None => vec![
            keyed("name", centered(l10n.activity_unknown_process(i64::from(went.key.pid)))),
            keyed("why", secondary(l10n.activity_unknown_process_why(), *palette)),
        ],
    };
    let head = line(
        format::clock(went.exit.at),
        icon!(went).size(size::Icon).build_element(),
        spaced(parts),
        secondary(went.lived.map(|lived| lasted(l10n, lived)).unwrap_or_default(), *palette),
        *palette,
    );
    let body = if item.open {
        StackPanel::new().children((head, went_facts(went, l10n, *palette, Line::Indent))).into()
    } else {
        head
    };
    pressed(body, went.key, on_press)
}

fn series_names(series: &Series, l10n: &L10n) -> String {
    series
        .names
        .iter()
        .map(|(name, count)| l10n.activity_series_name(name.to_string(), *count as i64))
        .collect::<Vec<_>>()
        .join(&l10n.activity_series_names_separator())
}

fn member_view(came: &Came, l10n: &L10n, palette: Palette) -> View {
    let trailing = match &came.exit {
        Some(exit) => lasted(l10n, exit.lived),
        None => l10n.activity_still_running(),
    };
    Border::new()
        .margin(Thickness::new(Line::Icon, 0.0, 0.0, 0.0))
        .content(line(
            format::clock(came.at),
            icon!(came).size(size::Icon).build_element(),
            centered(came.name.to_string()),
            secondary(trailing, palette),
            palette,
        ))
        .into()
}

fn series_view(item: &Item, series: &Series) -> View {
    let Item { l10n, palette, on_press, open, .. } = item;
    let open = *open;
    let chevron = if open {
        icon!(chevron_down_regular).size(size::Icon).build_element()
    } else {
        icon!(chevron_right_regular).size(size::Icon).build_element()
    };
    let head = line(
        format::clock(series.at),
        icon!(burst).size(size::Icon).build_element(),
        spaced(vec![
            keyed("chevron", chevron),
            keyed("launcher", centered(series.launcher.to_string())),
            keyed("names", secondary(series_names(series, l10n), *palette)),
        ]),
        secondary(
            l10n.activity_series_summary(series.count as i64, series.went as i64),
            *palette,
        ),
        *palette,
    );
    let body = if open {
        let members: Vec<KeyedView> = series
            .members
            .iter()
            .map(|came| keyed(key(came.key), member_view(came, l10n, *palette)))
            .collect();
        StackPanel::new().children((head, StackPanel::new().keyed_children(members))).into()
    } else {
        head
    };
    pressed(body, series.key, on_press)
}

fn row_view(item: &Item) -> View {
    match &item.row {
        ActivityRow::Came(came) => came_view(item, came),
        ActivityRow::Went(went) => went_view(item, went),
        ActivityRow::Series(series) => series_view(item, series),
    }
}
