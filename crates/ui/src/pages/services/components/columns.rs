use std::fmt::Write;
use std::rc::Rc;

use app_contracts::features::agents::WindowsServiceState;
use app_contracts::features::services::{ServiceColumn, ServiceRow};
use table::model::{Cell, Icon, Tone};

use crate::l10n::L10n;
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

fn text_column(
    id: ServiceColumn,
    header: String,
    width: f64,
    write: impl Fn(&ServiceRow, &mut String) + 'static,
) -> Column {
    ColumnSpec::painted(id, header, width, move |row: &ServiceRow, cell: &mut Cell| {
        write(row, &mut cell.text);
        cell.dim = !row.is_running();
    })
}

pub(crate) fn build_columns(l10n: &L10n) -> Vec<Column> {
    let labels: Rc<[(WindowsServiceState, String)]> =
        STATES.iter().map(|&state| (state, state_label(l10n, state))).collect();
    let icon = guicons::icon_data!(gears).svg_bytes().map(Icon::Svg);
    vec![
        ColumnSpec::painted(ServiceColumn::Name, l10n.services_col_name(), 260.0, move |row: &ServiceRow, cell: &mut Cell| {
            cell.icon = icon.clone();
            cell.text.push_str(&row.display_name);
            cell.dim = !row.is_running();
        })
        .sortable(),
        ColumnSpec::painted(ServiceColumn::Status, l10n.services_col_status(), 90.0, move |row: &ServiceRow, cell: &mut Cell| {
            if let Some((_, label)) = labels.iter().find(|(state, _)| *state == row.state) {
                cell.text.push_str(label);
            }
            if row.is_running() {
                cell.tone = Tone::Success;
            }
            cell.dim = !row.is_running();
        })
        .sortable(),
        text_column(ServiceColumn::Pid, l10n.services_col_pid(), 70.0, |row, text| {
            if row.pid != 0 {
                let _ = write!(text, "{}", row.pid);
            }
        })
        .sortable(),
        text_column(ServiceColumn::Group, l10n.services_col_group(), 120.0, |row, text| text.push_str(&row.group))
            .sortable(),
        text_column(ServiceColumn::Description, l10n.services_col_description(), 320.0, |row, text| {
            text.push_str(&row.description)
        }),
    ]
}
