use app_contracts::features::activity::{Came, Launcher, Went};
use guinea::winui::MarkExt;
use windows_reactor::{Grid, GridLength, KeyedView, TextWrapping, Thickness, View, keyed};

use super::super::marks::ActivityMark;
use super::lasted::lasted;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{Palette, space};
use crate::widgets::text::caption;

fn fact(at: usize, label: String, value: String, palette: Palette) -> [KeyedView; 2] {
    [
        keyed(
            format!("{at}/label"),
            caption(label)
                .grid_row(at as i32)
                .grid_column(0)
                .foreground(palette.secondary_text)
                .margin(Thickness::new(0.0, 0.0, space::Section, space::Compact)),
        ),
        keyed(
            format!("{at}/value"),
            caption(value)
                .grid_row(at as i32)
                .grid_column(1)
                .text_wrapping(TextWrapping::Wrap)
                .is_text_selection_enabled(true)
                .margin(Thickness::new(0.0, 0.0, 0.0, space::Compact)),
        ),
    ]
}

fn or_unknown(value: &str, l10n: &L10n) -> String {
    if value.is_empty() {
        l10n.activity_fact_unknown()
    } else {
        value.to_string()
    }
}

pub fn facts(came: &Came, l10n: &L10n, palette: Palette, indent: f64) -> View {
    let mut lines = vec![
        (
            l10n.activity_fact_command_line(),
            or_unknown(&came.command_line, l10n),
        ),
        (
            l10n.activity_fact_launched_by(),
            came.chain
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
                .join(&l10n.activity_chain_separator()),
        ),
    ];
    if let Launcher::Task(task) = &came.launcher {
        lines.push((
            l10n.activity_fact_task(),
            l10n.activity_fact_task_value(task.name.to_string(), task.path.to_string()),
        ));
    }
    if !came.parent_services.is_empty() {
        lines.push((
            l10n.activity_fact_parent_services(),
            came.parent_services
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
                .join(&l10n.activity_services_separator()),
        ));
    }
    lines.push((
        l10n.activity_fact_folder(),
        or_unknown(&came.working_dir, l10n),
    ));
    let user = or_unknown(&came.user, l10n);
    let session = i64::from(came.session_id);
    lines.push((
        l10n.activity_fact_user(),
        if came.elevated == Some(true) {
            l10n.activity_fact_user_elevated(user, session)
        } else {
            l10n.activity_fact_user_value(user, session)
        },
    ));
    if let Some(exit) = &came.exit {
        lines.push((
            l10n.activity_fact_exit(),
            l10n.activity_fact_exit_value(
                format::clock(exit.at),
                i64::from(exit.code),
                lasted(l10n, exit.lived),
            ),
        ));
    }
    table(lines, palette, indent)
}

pub fn went_facts(went: &Went, l10n: &L10n, palette: Palette, indent: f64) -> View {
    let exit = &went.exit;
    let value = match went.lived {
        Some(lived) => l10n.activity_fact_exit_value(
            format::clock(exit.at),
            i64::from(exit.code),
            lasted(l10n, lived),
        ),
        None => l10n.activity_fact_exit_code(format::clock(exit.at), i64::from(exit.code)),
    };
    table(vec![(l10n.activity_fact_exit(), value)], palette, indent)
}

fn table(lines: Vec<(String, String)>, palette: Palette, indent: f64) -> View {
    let rows = vec![GridLength::Auto; lines.len()];
    let cells: Vec<KeyedView> = lines
        .into_iter()
        .enumerate()
        .flat_map(|(at, (label, value))| fact(at, label, value, palette))
        .collect();
    Grid::new()
        .mark(ActivityMark::Facts)
        .columns([GridLength::Auto, GridLength::Star(1.0)])
        .rows(rows)
        .margin(Thickness::new(
            indent + space::Control,
            space::Compact,
            0.0,
            space::Control,
        ))
        .keyed_children(cells)
        .into()
}
