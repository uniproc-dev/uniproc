use std::fmt::Write;
use std::rc::Rc;

use app_contracts::features::agents::{WindowsServiceState, WindowsServiceStartType};
use app_contracts::features::services::{ServiceColumn, ServiceRow};
use windows_reactor::{Grid, GridLength, Thickness, TooltipExt, VerticalAlignment, View};
use guinea::winui::MarkExt;
use super::column_layout::ColumnLayout;
use crate::pages::services::ServicesMark;
use crate::widgets::selection::selection_indicator;

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

pub(crate) fn column_label(l10n: &L10n, column: ServiceColumn) -> String {
    match column {
        ServiceColumn::Name => l10n.services_col_name(),
        ServiceColumn::Status => l10n.services_col_status(),
        ServiceColumn::Pid => l10n.services_col_pid(),
        ServiceColumn::Group => l10n.services_col_group(),
        ServiceColumn::Description => l10n.services_col_description(),
        ServiceColumn::ServiceName => l10n.services_col_service_name(),
        ServiceColumn::ImagePath => l10n.services_col_image_path(),
        ServiceColumn::StartupType => l10n.services_col_startup_type(),
        ServiceColumn::Account => l10n.services_col_account(),
        ServiceColumn::ServiceType => l10n.services_col_service_type(),
    }
}

pub(crate) fn build_columns(l10n: &L10n, palette: Palette, layout: &ColumnLayout, selected: Option<&str>) -> Vec<Column> {
    let labels: Rc<[(WindowsServiceState, String)]> =
        STATES.iter().map(|&state| (state, state_label(l10n, state))).collect();
    let selected = selected.map(str::to_owned);
    ServiceColumn::ALL.into_iter().filter_map(|id| {
        let config = layout.config(id);
        if !config.visible { return None; }
        let label = column_label(l10n, id);
        let width = config.width as f64;
        let column = match id {
            ServiceColumn::Name => {
                let selected = selected.clone();
                column(id, label, width, palette, move |row| {
                    let content = Grid::new()
                        .columns([GridLength::Auto, GridLength::Star(1.0)])
                        .vertical_alignment(VerticalAlignment::Center)
                        .margin(Thickness::new(space::Cell, 0.0, 0.0, 0.0))
                        .children((
                            table_cell::service_icon(),
                            table_cell::cell_text(row.display_name.to_string())
                                .foreground(palette.primary_text)
                                .grid_column(1)
                                .margin(Thickness::new(space::Control, 0.0, 0.0, 0.0)),
                        ))
                        .tooltip(row.display_name.as_ref());
                    let content = table_cell::dimmed(content, !row.is_running());
                    let mut layers = vec![content];
                    if selected.as_deref() == Some(row.name.as_ref()) {
                        layers.push(selection_indicator().mark(ServicesMark::SelectedIndicator).into());
                    }
                    Grid::new().height(size::ProcessRow).children(layers).into()
                })
            }
            ServiceColumn::Status => {
                let labels = labels.clone();
                column(id, label, width, palette, move |row| {
                    let label = labels.iter().find(|(state, _)| *state == row.state)
                        .map(|(_, label)| label.clone()).unwrap_or_default();
                    let color = if row.is_running() { palette.success } else { palette.primary_text };
                    table_cell::dimmed(table_cell::cell_text(label.clone()).foreground(color).tooltip(label), !row.is_running())
                })
            }
            ServiceColumn::StartupType => {
                let l10n = l10n.clone();
                column(id, label, width, palette, move |row| {
                    let label = match row.startup_type {
                        WindowsServiceStartType::Unknown => l10n.services_state_unknown(),
                        WindowsServiceStartType::Boot => l10n.services_startup_boot(),
                        WindowsServiceStartType::System => l10n.services_startup_system(),
                        WindowsServiceStartType::Automatic if row.delayed_auto => l10n.services_startup_delayed(),
                        WindowsServiceStartType::Automatic => l10n.services_startup_automatic(),
                        WindowsServiceStartType::Manual => l10n.services_startup_manual(),
                        WindowsServiceStartType::Disabled => l10n.services_startup_disabled(),
                    };
                    table_cell::cell_text(label.clone()).foreground(palette.primary_text).tooltip(label).into()
                })
            }
            ServiceColumn::ServiceType => {
                let l10n = l10n.clone();
                column(id, label, width, palette, move |row| {
                    let label = if row.service_type & 0x40 != 0 { l10n.services_type_user() }
                        else if row.service_type & 0x20 != 0 { l10n.services_type_shared() }
                        else if row.service_type & 0x10 != 0 { l10n.services_type_own() }
                        else if row.service_type & 2 != 0 { l10n.services_type_file_system() }
                        else if row.service_type & 1 != 0 { l10n.services_type_kernel() }
                        else { l10n.services_state_unknown() };
                    table_cell::cell_text(label.clone()).foreground(palette.primary_text).tooltip(label).into()
                })
            }
            _ => text_column(id, label, width, palette, move |row, text| match id {
                ServiceColumn::Pid if row.pid != 0 => { let _ = write!(text, "{}", row.pid); }
                ServiceColumn::Group => text.push_str(&row.group),
                ServiceColumn::Description => text.push_str(&row.description),
                ServiceColumn::ServiceName => text.push_str(&row.name),
                ServiceColumn::ImagePath => text.push_str(&row.image_path),
                ServiceColumn::Account => text.push_str(&row.account),
                _ => {}
            }),
        };
        Some(column.sortable())
    }).collect()
}
