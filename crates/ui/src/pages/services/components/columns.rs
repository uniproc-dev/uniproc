use std::fmt::Write;
use std::rc::Rc;

use app_contracts::features::agents::WindowsServiceState;
use app_contracts::features::services::{ServiceColumn, ServiceRow};
use windows_reactor::{Grid, GridLength, Thickness, TooltipExt, VerticalAlignment, View};

use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::table_cell;
use crate::widgets::table::ColumnSpec;

fn state_label(l10n: &L10n, state: WindowsServiceState) -> String {
    match state {
        WindowsServiceState::Unknown => l10n.services_state_unknown(),
        WindowsServiceState::Stopped => l10n.services_state_stopped(),
        WindowsServiceState::StartPending => l10n.services_state_start_pending(),
        WindowsServiceState::StopPending => l10n.services_state_stop_pending(),
        WindowsServiceState::Running => l10n.services_state_running(),
        WindowsServiceState::ContinuePending => l10n.services_state_continue_pending(),
        WindowsServiceState::PausePending => l10n.services_state_pause_pending(),
        WindowsServiceState::Paused => l10n.services_state_paused(),
    }
}

const STATES: [WindowsServiceState; 8] = [
    WindowsServiceState::Unknown,
    WindowsServiceState::Stopped,
    WindowsServiceState::StartPending,
    WindowsServiceState::StopPending,
    WindowsServiceState::Running,
    WindowsServiceState::ContinuePending,
    WindowsServiceState::PausePending,
    WindowsServiceState::Paused,
];

type Column = ColumnSpec<ServiceRow, ServiceColumn>;

fn column(id: ServiceColumn, label: String, width: f64, palette: Palette, cell: impl Fn(&ServiceRow) -> View + 'static) -> Column {
    ColumnSpec::new_with_header(id, move || {
        Grid::new()
            .height(size::TableHeader)
            .children((table_cell::cell_text(label.clone())
                .foreground(palette.tertiary_text)
                .vertical_alignment(VerticalAlignment::Bottom),))
            .into()
    }, width, cell)
}

fn text_column(
    id: ServiceColumn,
    header: String,
    width: f64,
    palette: Palette,
    write: impl Fn(&ServiceRow, &mut String) + 'static,
) -> Column {
    column(id, header, width, palette, move |row: &ServiceRow| {
        let mut text = String::new();
        write(row, &mut text);
        let content = table_cell::cell_text(text.clone()).foreground(palette.primary_text).tooltip(text);
        table_cell::dimmed(content, !row.is_running())
    })
}

pub(crate) fn build_columns(l10n: &L10n, palette: Palette) -> Vec<Column> {
    let labels: Rc<[(WindowsServiceState, String)]> =
        STATES.iter().map(|&state| (state, state_label(l10n, state))).collect();
    vec![
        column(ServiceColumn::Name, l10n.services_col_name(), 260.0, palette, move |row: &ServiceRow| {
            let content = Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0)])
                .vertical_alignment(VerticalAlignment::Center)
                .children((
                    table_cell::service_icon(),
                    table_cell::cell_text(row.display_name.to_string())
                        .foreground(palette.primary_text)
                        .grid_column(1)
                        .margin(Thickness::new(space::Control, 0.0, 0.0, 0.0)),
                ))
                .tooltip(row.display_name.as_ref());
            table_cell::dimmed(content, !row.is_running())
        })
        .sortable(),
        column(ServiceColumn::Status, l10n.services_col_status(), 90.0, palette, move |row: &ServiceRow| {
            let label = labels
                .iter()
                .find(|(state, _)| *state == row.state)
                .map(|(_, label)| label.clone())
                .unwrap_or_default();
            let color = if row.is_running() { palette.success } else { palette.primary_text };
            let content = table_cell::cell_text(label.clone()).foreground(color).tooltip(label);
            table_cell::dimmed(content, !row.is_running())
        })
        .sortable(),
        text_column(ServiceColumn::Pid, l10n.services_col_pid(), 70.0, palette, |row, text| {
            if row.pid != 0 {
                let _ = write!(text, "{}", row.pid);
            }
        })
        .sortable(),
        text_column(ServiceColumn::Group, l10n.services_col_group(), 120.0, palette, |row, text| text.push_str(&row.group))
            .sortable(),
        text_column(ServiceColumn::Description, l10n.services_col_description(), 320.0, palette, |row, text| {
            text.push_str(&row.description)
        }),
    ]
}
