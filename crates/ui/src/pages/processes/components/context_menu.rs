use std::sync::Arc;

use app_contracts::features::agents::ProcessPriority;
use app_contracts::features::processes::{
    GroupCommand, PinnedProcess, ProcessColumn, ProcessCommand, ProcessRow, ProcessStatus, ProcessWindow,
    WindowCommand,
};
use guicons::icon;
use windows_reactor::{Border, Callback, View};

use super::super::marks::ProcessesMark;
use super::columns::column_label;
use crate::l10n::L10n;
use crate::theme::{size, Palette};
use crate::widgets::popup_menu::{popup_menu, MenuLine, PopupMenu};

#[derive(Clone, PartialEq, Debug)]
pub enum MenuTarget {
    Process(ProcessRow),
    Group { leader: ProcessRow },
    Window { window: ProcessWindow },
    Absent { image: ProcessRow },
    Columns,
}

impl MenuTarget {
    pub(crate) fn pin(&self) -> Option<(Arc<str>, PinnedProcess)> {
        match self {
            Self::Process(row) | Self::Group { leader: row } | Self::Absent { image: row } => Some((
                row.name.clone(),
                PinnedProcess {
                    exe_path: row.exe_path.to_string(),
                    package_full_name: row.package_full_name.to_string(),
                    display_name: row.display_name.to_string(),
                },
            )),
            Self::Window { .. } | Self::Columns => None,
        }
    }

    pub(crate) fn image(&self) -> Option<&ProcessRow> {
        match self {
            Self::Absent { image } => Some(image),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct OpenMenu {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) target: MenuTarget,
    pub(crate) priority: bool,
}

impl OpenMenu {
    pub(crate) fn at(x: f64, y: f64, target: MenuTarget) -> Self {
        Self {
            x,
            y,
            target,
            priority: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MenuCommand {
    TogglePin,
    EndTask,
    Group(GroupCommand),
    ShowPriority,
    Process(ProcessCommand),
    Window { handle: isize, command: WindowCommand },
    ToggleColumn(ProcessColumn),
    OpenSettings,
}

type Line = MenuLine<ProcessesMark, MenuCommand>;

fn needs_path(line: Line, row: &ProcessRow) -> Line {
    line.enabled_if(!row.exe_path.is_empty())
}

fn file_lines(row: &ProcessRow, l10n: &L10n) -> Vec<Line> {
    vec![
        needs_path(
            Line::entry(
                ProcessesMark::MenuOpenFileLocation,
                icon!(folder).size(size::Icon).build_element(),
                l10n.processes_menu_open_file_location(),
                MenuCommand::Process(ProcessCommand::OpenFileLocation),
            ),
            row,
        ),
        Line::entry(
            ProcessesMark::MenuSearchOnline,
            icon!(search).size(size::Icon).build_element(),
            l10n.processes_menu_search_online(),
            MenuCommand::Process(ProcessCommand::SearchOnline),
        ),
        needs_path(
            Line::entry(
                ProcessesMark::MenuProperties,
                icon!(info).size(size::Icon).build_element(),
                l10n.processes_menu_properties(),
                MenuCommand::Process(ProcessCommand::Properties),
            ),
            row,
        ),
    ]
}

fn pin_lines(pinned: bool, l10n: &L10n) -> [Line; 2] {
    let pin = if pinned {
        Line::entry(
            ProcessesMark::MenuUnpin,
            icon!(pin_off).size(size::Icon).build_element(),
            l10n.processes_menu_unpin(),
            MenuCommand::TogglePin,
        )
    } else {
        Line::entry(
            ProcessesMark::MenuPin,
            icon!(pin).size(size::Icon).build_element(),
            l10n.processes_menu_pin(),
            MenuCommand::TogglePin,
        )
    };
    [pin, Line::Separator]
}

fn is_suspended(row: &ProcessRow) -> bool {
    row.details.status == ProcessStatus::Suspended
}

fn group_lines(leader: &ProcessRow, members: &[ProcessRow], pinned: bool, l10n: &L10n) -> Vec<Line> {
    let count = members.len() as i64;
    let acting: Vec<&ProcessRow> = members.iter().filter(|row| row.takes_actions()).collect();
    let mut lines = Vec::from(pin_lines(pinned, l10n));
    lines.push(
        Line::entry(
            ProcessesMark::MenuEndGroup,
            icon!(prohibited).size(size::Icon).build_element(),
            l10n.processes_menu_end_group(count),
            MenuCommand::Group(GroupCommand::End),
        )
        .enabled_if(!acting.is_empty()),
    );
    if acting.iter().any(|row| !is_suspended(row)) {
        lines.push(Line::entry(
            ProcessesMark::MenuSuspendGroup,
            icon!(pause).size(size::Icon).build_element(),
            l10n.processes_menu_suspend_group(count),
            MenuCommand::Group(GroupCommand::Suspend),
        ));
    }
    if acting.iter().any(|row| is_suspended(row)) {
        lines.push(Line::entry(
            ProcessesMark::MenuResumeGroup,
            icon!(play).size(size::Icon).build_element(),
            l10n.processes_menu_resume_group(count),
            MenuCommand::Group(GroupCommand::Resume),
        ));
    }
    lines.push(Line::Separator);
    lines.extend(file_lines(leader, l10n));
    lines
}

fn image_lines(image: &ProcessRow, pinned: bool, l10n: &L10n) -> Vec<Line> {
    let mut lines = Vec::from(pin_lines(pinned, l10n));
    lines.extend(file_lines(image, l10n));
    lines
}

fn process_lines(row: &ProcessRow, pinned: bool, l10n: &L10n) -> Vec<Line> {
    let actions = row.takes_actions();
    let pause = if is_suspended(row) {
        Line::entry(
            ProcessesMark::MenuResume,
            icon!(play).size(size::Icon).build_element(),
            l10n.processes_menu_resume(),
            MenuCommand::Process(ProcessCommand::Resume),
        )
    } else {
        Line::entry(
            ProcessesMark::MenuSuspend,
            icon!(pause).size(size::Icon).build_element(),
            l10n.processes_menu_suspend(),
            MenuCommand::Process(ProcessCommand::Suspend),
        )
    };
    let mut lines = Vec::from(pin_lines(pinned, l10n));
    lines.extend(
        [
            Line::entry(
                ProcessesMark::MenuEndTask,
                icon!(prohibited).size(size::Icon).build_element(),
                l10n.processes_menu_end_task(),
                MenuCommand::EndTask,
            ),
            pause,
            Line::entry(
                ProcessesMark::MenuPriority,
                icon!(top_speed).size(size::Icon).build_element(),
                l10n.processes_menu_priority(),
                MenuCommand::ShowPriority,
            ),
        ]
        .map(|line| line.enabled_if(actions)),
    );
    lines.push(Line::Separator);
    lines.extend(file_lines(row, l10n));
    lines
}

fn priority_lines(current: Option<ProcessPriority>, l10n: &L10n) -> Vec<Line> {
    [
        (ProcessPriority::Realtime, ProcessesMark::MenuPriorityRealtime, l10n.processes_menu_priority_realtime()),
        (ProcessPriority::High, ProcessesMark::MenuPriorityHigh, l10n.processes_menu_priority_high()),
        (
            ProcessPriority::AboveNormal,
            ProcessesMark::MenuPriorityAboveNormal,
            l10n.processes_menu_priority_above_normal(),
        ),
        (ProcessPriority::Normal, ProcessesMark::MenuPriorityNormal, l10n.processes_menu_priority_normal()),
        (
            ProcessPriority::BelowNormal,
            ProcessesMark::MenuPriorityBelowNormal,
            l10n.processes_menu_priority_below_normal(),
        ),
        (ProcessPriority::Idle, ProcessesMark::MenuPriorityLow, l10n.processes_menu_priority_low()),
    ]
    .into_iter()
    .map(|(priority, mark, label)| {
        let icon = if current == Some(priority) {
            icon!(checkmark).size(size::Icon).build_element()
        } else {
            Border::new().width(size::Icon).height(size::Icon).into()
        };
        Line::entry(mark, icon, label, MenuCommand::Process(ProcessCommand::Priority(priority)))
    })
    .collect()
}

fn window_lines(handle: isize, l10n: &L10n) -> Vec<Line> {
    let command = |command| MenuCommand::Window { handle, command };
    vec![
        Line::entry(
            ProcessesMark::MenuSwitchTo,
            icon!(open).size(size::Icon).build_element(),
            l10n.processes_menu_switch_to(),
            command(WindowCommand::SwitchTo),
        ),
        Line::entry(
            ProcessesMark::MenuMinimize,
            icon!(minimize).size(size::Icon).build_element(),
            l10n.processes_menu_minimize(),
            command(WindowCommand::Minimize),
        ),
        Line::entry(
            ProcessesMark::MenuMaximize,
            icon!(maximize).size(size::Icon).build_element(),
            l10n.processes_menu_maximize(),
            command(WindowCommand::Maximize),
        ),
        Line::Separator,
        Line::entry(
            ProcessesMark::MenuCloseWindow,
            icon!(dismiss).size(size::Icon).build_element(),
            l10n.processes_menu_close_window(),
            command(WindowCommand::Close),
        ),
    ]
}

fn column_mark(column: ProcessColumn) -> Option<ProcessesMark> {
    match column {
        ProcessColumn::Cpu => Some(ProcessesMark::MenuColumnCpu),
        ProcessColumn::Memory => Some(ProcessesMark::MenuColumnMemory),
        ProcessColumn::Net => Some(ProcessesMark::MenuColumnNet),
        ProcessColumn::Disk => Some(ProcessesMark::MenuColumnDisk),
        ProcessColumn::Gpu => Some(ProcessesMark::MenuColumnGpu),
        ProcessColumn::GpuMemory => Some(ProcessesMark::MenuColumnGpuMemory),
        ProcessColumn::Name
        | ProcessColumn::Pid
        | ProcessColumn::ProcessName
        | ProcessColumn::Status
        | ProcessColumn::Publisher
        | ProcessColumn::User
        | ProcessColumn::CommandLine
        | ProcessColumn::ImagePath
        | ProcessColumn::GpuEngine
        | ProcessColumn::Platform
        | ProcessColumn::Elevated
        | ProcessColumn::Isolation => None,
    }
}

fn column_lines(columns: &[(ProcessColumn, bool)], l10n: &L10n) -> Vec<Line> {
    let mut lines: Vec<Line> = columns
        .iter()
        .filter_map(|&(column, visible)| {
            let mark = column_mark(column)?;
            let icon = if visible {
                icon!(checkmark).size(size::Icon).build_element()
            } else {
                Border::new().width(size::Icon).height(size::Icon).into()
            };
            Some(Line::entry(mark, icon, column_label(l10n, column), MenuCommand::ToggleColumn(column)))
        })
        .collect();
    lines.push(Line::Separator);
    lines.push(Line::entry(
        ProcessesMark::MenuMoreColumns,
        icon!(more_horizontal).size(size::Icon).build_element(),
        l10n.processes_menu_more_columns(),
        MenuCommand::OpenSettings,
    ));
    lines
}

pub(crate) struct MenuInputs<'a> {
    pub(crate) l10n: &'a L10n,
    pub(crate) palette: Palette,
    pub(crate) pinned: bool,
    pub(crate) members: &'a [ProcessRow],
    pub(crate) columns: Vec<(ProcessColumn, bool)>,
    pub(crate) on_command: Callback<MenuCommand>,
    pub(crate) on_dismiss: Callback<()>,
}

pub(crate) fn context_menu(menu: &OpenMenu, inputs: MenuInputs<'_>) -> View {
    let MenuInputs {
        l10n,
        palette,
        pinned,
        members,
        columns,
        on_command,
        on_dismiss,
    } = inputs;
    let lines = match &menu.target {
        MenuTarget::Process(row) if menu.priority => priority_lines(row.details.priority, l10n),
        MenuTarget::Process(row) => process_lines(row, pinned, l10n),
        MenuTarget::Group { leader } => group_lines(leader, members, pinned, l10n),
        MenuTarget::Window { window } => window_lines(window.handle, l10n),
        MenuTarget::Absent { image } => image_lines(image, pinned, l10n),
        MenuTarget::Columns => column_lines(&columns, l10n),
    };

    popup_menu(PopupMenu {
        x: menu.x,
        y: menu.y,
        lines,
        card: ProcessesMark::Menu,
        backdrop: ProcessesMark::MenuBackdrop,
        palette,
        on_command,
        on_dismiss,
    })
}
