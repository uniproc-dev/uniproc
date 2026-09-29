use std::sync::Arc;

use app_contracts::features::processes::{
    PinnedProcess, ProcessColumn, ProcessCommand, ProcessRow, ProcessWindow, WindowCommand,
};
use guicons::icon;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, ChildrenControl, Color, ContentControl, CornerRadius,
    Grid, GridChildExt, GridLength, HorizontalAlignment, KeyedView, LayoutControl, Orientation,
    PointerEventInfo, StackPanel, Thickness, VerticalAlignment, View,
};

use guinea::winui::MarkExt;

use super::super::marks::ProcessesMark;
use super::columns::column_label;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::separator;
use crate::widgets::text::text;

struct Menu;

#[expect(non_upper_case_globals)]
impl Menu {
    const Width: f64 = 248.0;
    const ShadowDrop: f64 = 2.0;
    const Backdrop: Color = Color::argb(0, 0, 0, 0);
}

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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MenuCommand {
    TogglePin,
    EndTask,
    Process(ProcessCommand),
    Window { handle: isize, command: WindowCommand },
    ToggleColumn(ProcessColumn),
    OpenSettings,
}

struct Entry {
    mark: ProcessesMark,
    icon: View,
    label: String,
    enabled: bool,
    command: MenuCommand,
}

enum Line {
    Entry(Entry),
    Separator,
}

fn entry(mark: ProcessesMark, icon: View, label: String, command: MenuCommand) -> Line {
    Line::Entry(Entry {
        mark,
        icon,
        label,
        enabled: true,
        command,
    })
}

fn enabled_if(line: Line, enabled: bool) -> Line {
    match line {
        Line::Entry(entry) => Line::Entry(Entry { enabled, ..entry }),
        Line::Separator => Line::Separator,
    }
}

fn needs_path(line: Line, row: &ProcessRow) -> Line {
    enabled_if(line, !row.exe_path.is_empty())
}

fn file_lines(row: &ProcessRow, l10n: &L10n) -> Vec<Line> {
    vec![
        needs_path(
            entry(
                ProcessesMark::MenuOpenFileLocation,
                icon!(folder).size(size::Icon).build_element(),
                l10n.processes_menu_open_file_location(),
                MenuCommand::Process(ProcessCommand::OpenFileLocation),
            ),
            row,
        ),
        entry(
            ProcessesMark::MenuSearchOnline,
            icon!(search).size(size::Icon).build_element(),
            l10n.processes_menu_search_online(),
            MenuCommand::Process(ProcessCommand::SearchOnline),
        ),
        needs_path(
            entry(
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
        entry(
            ProcessesMark::MenuUnpin,
            icon!(pin_off).size(size::Icon).build_element(),
            l10n.processes_menu_unpin(),
            MenuCommand::TogglePin,
        )
    } else {
        entry(
            ProcessesMark::MenuPin,
            icon!(pin).size(size::Icon).build_element(),
            l10n.processes_menu_pin(),
            MenuCommand::TogglePin,
        )
    };
    [pin, Line::Separator]
}

fn group_lines(leader: &ProcessRow, pinned: bool, l10n: &L10n) -> Vec<Line> {
    let mut lines = Vec::from(pin_lines(pinned, l10n));
    lines.extend(file_lines(leader, l10n));
    lines
}

fn process_lines(row: &ProcessRow, pinned: bool, l10n: &L10n) -> Vec<Line> {
    let actions = row.category.takes_actions();
    let mut lines = Vec::from(pin_lines(pinned, l10n));
    lines.extend(
        [
            entry(
                ProcessesMark::MenuEndTask,
                icon!(prohibited).size(size::Icon).build_element(),
                l10n.processes_menu_end_task(),
                MenuCommand::EndTask,
            ),
            entry(
                ProcessesMark::MenuSuspend,
                icon!(pause).size(size::Icon).build_element(),
                l10n.processes_menu_suspend(),
                MenuCommand::Process(ProcessCommand::Suspend),
            ),
            entry(
                ProcessesMark::MenuResume,
                icon!(play).size(size::Icon).build_element(),
                l10n.processes_menu_resume(),
                MenuCommand::Process(ProcessCommand::Resume),
            ),
        ]
        .map(|line| enabled_if(line, actions)),
    );
    lines.push(Line::Separator);
    lines.extend(file_lines(row, l10n));
    lines
}

fn window_lines(handle: isize, l10n: &L10n) -> Vec<Line> {
    let command = |command| MenuCommand::Window { handle, command };
    vec![
        entry(
            ProcessesMark::MenuSwitchTo,
            icon!(open).size(size::Icon).build_element(),
            l10n.processes_menu_switch_to(),
            command(WindowCommand::SwitchTo),
        ),
        entry(
            ProcessesMark::MenuMinimize,
            icon!(minimize).size(size::Icon).build_element(),
            l10n.processes_menu_minimize(),
            command(WindowCommand::Minimize),
        ),
        entry(
            ProcessesMark::MenuMaximize,
            icon!(maximize).size(size::Icon).build_element(),
            l10n.processes_menu_maximize(),
            command(WindowCommand::Maximize),
        ),
        Line::Separator,
        entry(
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
            Some(entry(mark, icon, column_label(l10n, column), MenuCommand::ToggleColumn(column)))
        })
        .collect();
    lines.push(Line::Separator);
    lines.push(entry(
        ProcessesMark::MenuMoreColumns,
        icon!(more_horizontal).size(size::Icon).build_element(),
        l10n.processes_menu_more_columns(),
        MenuCommand::OpenSettings,
    ));
    lines
}

fn line_view(line: Line, on_command: &Callback<MenuCommand>, palette: Palette) -> View {
    match line {
        Line::Separator => separator(palette)
            .margin(Thickness::xy(0.0, space::Compact))
            .into(),
        Line::Entry(entry) => {
            let on_command = on_command.clone();
            let command = entry.command;
            Button::new()
                .mark(entry.mark)
                .style(ButtonStyle::Subtle)
                .is_enabled(entry.enabled)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .horizontal_content_alignment(HorizontalAlignment::Left)
                .on_click(move || {
                    let _ = on_command.call(command);
                })
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

pub(crate) struct MenuInputs<'a> {
    pub(crate) l10n: &'a L10n,
    pub(crate) palette: Palette,
    pub(crate) pinned: bool,
    pub(crate) columns: Vec<(ProcessColumn, bool)>,
    pub(crate) on_command: Callback<MenuCommand>,
    pub(crate) on_dismiss: Callback<()>,
}

pub(crate) fn context_menu(menu: &OpenMenu, inputs: MenuInputs<'_>) -> View {
    let MenuInputs {
        l10n,
        palette,
        pinned,
        columns,
        on_command,
        on_dismiss,
    } = inputs;
    let lines = match &menu.target {
        MenuTarget::Process(row) => process_lines(row, pinned, l10n),
        MenuTarget::Group { leader } => group_lines(leader, pinned, l10n),
        MenuTarget::Window { window } => window_lines(window.handle, l10n),
        MenuTarget::Absent { image } => group_lines(image, pinned, l10n),
        MenuTarget::Columns => column_lines(&columns, l10n),
    };

    let items = View::keyed_fragment(
        lines
            .into_iter()
            .map(|line| line_view(line, &on_command, palette))
            .enumerate()
            .map(|(at, view)| KeyedView::new(at, view)),
    );

    let card = Border::new()
        .mark(ProcessesMark::Menu)
        .grid_row(1)
        .grid_column(1)
        .width(Menu::Width)
        .background(palette.menu_fill)
        .border_brush(palette.menu_stroke)
        .border_thickness(Thickness::uniform(space::Hairline))
        .corner_radius(radius::Overlay)
        .padding(Thickness::uniform(space::Compact))
        .content(StackPanel::new().children((items,)));

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
            Border::new()
                .grid_row(0)
                .grid_column(0)
                .width(menu.x)
                .height(menu.y),
            shadow,
            card,
        ));

    let backdrop = Border::new()
        .mark(ProcessesMark::MenuBackdrop)
        .background(Menu::Backdrop)
        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
            let _ = on_dismiss.call(());
        }));

    Grid::new().children((backdrop, placed)).into()
}
