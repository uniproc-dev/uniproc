use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{ActivityRow, Burst, Came, Launcher, Went};
use app_contracts::features::agents::ProcessInstance;
use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength, HorizontalAlignment,
    ItemsRepeater, LayoutControl, Orientation, PointerEventInfo, StackPanel, TextTrimming, TextWrapping, Thickness,
    VerticalAlignment, View, VirtualSource,
};

use super::super::marks::ActivityMark;
use super::facts::facts;
use super::lasted::lasted;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, size, space, Palette};
use crate::widgets::text::{caption, text};

struct Line;

#[expect(non_upper_case_globals)]
impl Line {
    const Time: f64 = 64.0;
    const Icon: f64 = 24.0;
}

pub struct Rows {
    pub rows: Rc<[ActivityRow]>,
    pub expanded: Rc<HashSet<ProcessInstance>>,
    pub l10n: L10n,
    pub palette: Palette,
    pub on_toggle: Callback<ProcessInstance>,
}

pub fn rows(list: Rows) -> View {
    let len = list.rows.len();
    let list = Rc::new(list);
    ItemsRepeater::new()
        .mark(ActivityMark::Rows)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .virtual_source(VirtualSource::new(len as u64, len, |index| index, move |index| {
            match list.rows.get(index) {
                Some(row) => row_view(&list, row),
                None => Border::new().into(),
            }
        }))
        .into()
}

fn secondary(content: impl Into<String>, palette: Palette) -> View {
    caption(content)
        .foreground(palette.secondary_text)
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

fn line(at: String, glyph: View, main: View, trailing: View, palette: Palette) -> View {
    Grid::new()
        .columns([
            GridLength::Pixel(Line::Time),
            GridLength::Pixel(Line::Icon),
            GridLength::Star(1.0),
            GridLength::Auto,
        ])
        .min_height(size::ProcessRow)
        .children((
            caption(at)
                .grid_column(0)
                .foreground(palette.secondary_text)
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
}

fn toggled(content: impl Into<View>, key: ProcessInstance, on_toggle: &Callback<ProcessInstance>) -> View {
    let on_toggle = on_toggle.clone();
    Border::new()
        .background(Color::transparent())
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .padding(Thickness::xy(space::Cell, 0.0))
        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
            let _ = on_toggle.call(key);
        }))
        .content(content)
}

fn launcher_text(came: &Came, l10n: &L10n) -> Option<String> {
    match &came.launcher {
        Launcher::Task(task) => Some(l10n.activity_from_task(task.name.to_string())),
        Launcher::Process(name) => Some(l10n.activity_from(name.to_string())),
        Launcher::Unknown => None,
    }
}

fn came_main(came: &Came, l10n: &L10n, palette: Palette) -> View {
    let mut head: Vec<(String, View)> = vec![("name".into(), text(came.name.to_string()).into())];
    if let Some(from) = launcher_text(came, l10n) {
        head.push(("from".into(), secondary(from, palette)));
    }
    if came.first_seen {
        head.push((
            "first-seen".into(),
            caption(l10n.activity_first_seen())
                .foreground(accent_color())
                .vertical_alignment(VerticalAlignment::Center)
                .into(),
        ));
    }
    StackPanel::new()
        .children((
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((View::keyed_fragment(head),)),
            secondary(came.command_line.to_string(), palette),
        ))
}

fn came_trailing(came: &Came, l10n: &L10n, palette: Palette) -> View {
    let said = match &came.exit {
        Some(exit) => l10n.activity_went_after(lasted(l10n, exit.lived)),
        None => l10n.activity_still_running(),
    };
    secondary(said, palette)
}

fn came_view(list: &Rows, came: &Came) -> View {
    let Rows { l10n, palette, .. } = list;
    let open = list.expanded.contains(&came.key);
    let head = line(
        format::clock(came.at),
        icon!(came).size(size::Icon).build(),
        came_main(came, l10n, *palette),
        came_trailing(came, l10n, *palette),
        *palette,
    );
    let body = if open {
        StackPanel::new().children((head, facts(came, l10n, *palette, Line::Time + Line::Icon)))
    } else {
        head
    };
    toggled(body, came.key, &list.on_toggle)
}

fn went_view(list: &Rows, went: &Went) -> View {
    let Rows { l10n, palette, .. } = list;
    let mut main: Vec<(String, View)> = vec![("name".into(), text(went.name.to_string()).into())];
    if let Some(lived) = went.lived {
        main.push(("lived".into(), secondary(l10n.activity_went_lived(lasted(l10n, lived)), *palette)));
    }
    let head = line(
        format::clock(went.exit.at),
        icon!(went).size(size::Icon).build(),
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Control)
            .children((View::keyed_fragment(main),)),
        secondary(l10n.activity_went_code(i64::from(went.exit.code)), *palette),
        *palette,
    );
    toggled(head, went.key, &list.on_toggle)
}

fn burst_names(burst: &Burst, l10n: &L10n) -> String {
    burst
        .names
        .iter()
        .map(|(name, count)| l10n.activity_burst_name(name.to_string(), *count as i64))
        .collect::<Vec<_>>()
        .join(&l10n.activity_burst_names_separator())
}

fn member_view(came: &Came, l10n: &L10n, palette: Palette) -> View {
    let trailing = match &came.exit {
        Some(exit) => secondary(lasted(l10n, exit.lived), palette),
        None => secondary(l10n.activity_still_running(), palette),
    };
    Border::new()
        .margin(Thickness::new(Line::Icon, 0.0, 0.0, 0.0))
        .content(line(
            format::clock(came.at),
            icon!(came).size(size::Icon).build(),
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((
                    text(came.name.to_string()),
                    secondary(came.command_line.to_string(), palette),
                )),
            trailing,
            palette,
        ))
}

fn burst_view(list: &Rows, burst: &Burst) -> View {
    let Rows { l10n, palette, .. } = list;
    let open = list.expanded.contains(&burst.key);
    let chevron = if open {
        icon!(chevron_down_regular).size(size::Icon).build()
    } else {
        icon!(chevron_right_regular).size(size::Icon).build()
    };
    let head = line(
        format::clock(burst.at),
        icon!(burst).size(size::Icon).build(),
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Control)
            .children((chevron, text(burst.launcher.to_string()), secondary(burst_names(burst, l10n), *palette))),
        secondary(
            l10n.activity_burst_summary(
                burst.members.len() as i64,
                burst.went as i64,
                lasted(l10n, burst.lasted),
            ),
            *palette,
        ),
        *palette,
    );
    let body = if open {
        let members: Vec<(String, View)> = burst
            .members
            .iter()
            .map(|came| (format!("{}:{}", came.key.pid, came.key.sequence), member_view(came, l10n, *palette)))
            .collect();
        StackPanel::new().children((head, StackPanel::new().children((View::keyed_fragment(members),))))
    } else {
        head
    };
    toggled(body, burst.key, &list.on_toggle)
}

fn row_view(list: &Rows, row: &ActivityRow) -> View {
    match row {
        ActivityRow::Came(came) => came_view(list, came),
        ActivityRow::Went(went) => went_view(list, went),
        ActivityRow::Burst(burst) => burst_view(list, burst),
    }
}
