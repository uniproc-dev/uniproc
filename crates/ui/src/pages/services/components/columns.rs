use app_contracts::features::agents::WindowsServiceState;
use app_contracts::features::services::{ServiceColumn, ServiceRow};
use guicons::icon;
use guinea_widgets::table::ColumnSpec;
use windows_reactor::{
    Border, ChildrenControl, ContentControl, LayoutControl, Orientation, StackPanel, View,
};

use crate::l10n::L10n;
use crate::theme::{opacity, size, space, Palette};
use crate::widgets::table_cell;

fn service_icon() -> View {
    icon!(gears).size(size::Icon).build_element()
}

fn dimmed(content: impl Into<View>, running: bool) -> View {
    let opacity = if running { 1.0 } else { opacity::Stopped };
    Border::new().opacity(opacity).content(content)
}

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

type Column = ColumnSpec<ServiceRow, ServiceColumn>;

fn text_column(
    id: ServiceColumn,
    header: String,
    width: f64,
    read: impl Fn(&ServiceRow) -> String + 'static,
) -> Column {
    ColumnSpec::new(id, header, width, move |row: &ServiceRow| {
        dimmed(table_cell::cell_text(read(row)), row.is_running())
    })
}

pub(crate) fn build_columns(l10n: &L10n, palette: Palette) -> Vec<Column> {
    let status_l10n = l10n.clone();
    vec![
        ColumnSpec::new(ServiceColumn::Name, l10n.services_col_name(), 260.0, |row: &ServiceRow| {
            let content = StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((
                    service_icon(),
                    table_cell::cell_text(&*row.display_name),
                ));
            dimmed(content, row.is_running())
        })
        .sortable(),
        ColumnSpec::new(
            ServiceColumn::Status,
            l10n.services_col_status(),
            90.0,
            move |row: &ServiceRow| {
                let running = row.is_running();
                let label = table_cell::cell_text(state_label(&status_l10n, row.state));
                let label = if running {
                    label.foreground(palette.success)
                } else {
                    label
                };
                dimmed(label, running)
            },
        )
        .sortable(),
        text_column(ServiceColumn::Pid, l10n.services_col_pid(), 70.0, |row| {
            if row.pid == 0 {
                String::new()
            } else {
                row.pid.to_string()
            }
        })
        .sortable(),
        text_column(ServiceColumn::Group, l10n.services_col_group(), 120.0, |row| {
            row.group.to_string()
        })
        .sortable(),
        text_column(
            ServiceColumn::Description,
            l10n.services_col_description(),
            320.0,
            |row| row.description.to_string(),
        ),
    ]
}
