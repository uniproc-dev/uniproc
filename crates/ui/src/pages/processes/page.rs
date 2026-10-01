use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use amethystate::{Field, ReactiveMap};
use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::processes::{
    ColumnConfig, Deselect, DismissFailure, PinnedProcess, ProcessCategory, ProcessColumn, ProcessRow, ProcessesState, RunNewTask,
    RunProcessCommand,
    RunImageCommand, RunWindowCommand, Select, SelectLinux, Sort, Terminate, TerminateGroup,
};
use app_contracts::features::settings::Units;
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use crate::widgets::table::{table, Look, Reordered, Resized, SortState};
use windows_reactor::{
    Border, Callback, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, LayoutControl,
    PointerEventInfo, Orientation, StackPanel, Thickness,
    VerticalAlignment, View,
};

use super::components::column_layout::ColumnLayout;
use super::components::columns::{build_columns, ColumnInputs, GroupByType, NameCellActions};
use super::components::context_menu::{context_menu, MenuCommand, MenuInputs, MenuTarget, OpenMenu};
use super::components::grouping::{
    environment_key, flatten_for_display, group_members, highlight, keep_group_place, Child, DisplayRow, Grouping, GroupsCache, Pins,
    Held, Order, SectionId, SectionOrder, Selection, ViewState, WslRow,
};
use super::components::overlay::disconnected_overlay;
use super::components::section_drag::{self, Grab, Placement, SectionGesture};
use super::components::status::{status_bar, StatusCounts};
use super::marks::ProcessesMark;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::action_failure::action_failure;
use crate::widgets::button::{command_button, icon_button};
use crate::widgets::page::{loading, page_frame, page_title};
use crate::widgets::selection::SelectionMark;
use crate::widgets::text::text;

struct Header;

#[expect(non_upper_case_globals)]
impl Header {
    const DividerWidth: f64 = 1.0;
    const DividerHeight: f64 = 20.0;
    const CommandIcon: f64 = 18.0;
}

pub struct ProcessesSettingsMaps {
    pub columns: ReactiveMap<String, ColumnConfig>,
    pub column_order: ReactiveMap<String, u32>,
    pub collapsed_sections: ReactiveMap<String, bool>,
    pub group_by_type: Field<bool>,
    pub memory_as_percent: Field<bool>,
    pub pins: ReactiveMap<String, PinnedProcess>,
    pub section_order: ReactiveMap<String, u32>,
}

pub enum ProcessesMsg {
    Resized(Resized),
    Reordered(Reordered),
    ToggleGroup(String),
    ToggleProcess(u32),
    ToggleSection(SectionId),
    Section(SectionGesture),
    TogglePin(Arc<str>, PinnedProcess),
    ToggleGroupByType,
    SelectGroup(Option<u32>),
    MenuAnchor { x: f64, y: f64 },
    MenuFor(Option<MenuTarget>),
    MenuDismiss,
    ColumnMenu,
    ToggleColumn(ProcessColumn),
}

enum Press {
    Toggle(SectionId),
    Expand(String),
    Select(Selection),
    Nothing,
}

impl Press {
    fn of(d: &DisplayRow) -> Self {
        match (&d.section, &d.wsl) {
            (Some(section), _) => Self::Toggle(section.id),
            (None, Some(WslRow::Environment { pid_ns, .. })) => Self::Expand(environment_key(*pid_ns)),
            (None, Some(WslRow::Process { global_pid })) => Self::Select(Selection::Linux(*global_pid)),
            (None, None) if d.absent => Self::Nothing,
            (None, None) if d.has_children => Self::Select(Selection::Group(d.row.pid)),
            (None, None) => Self::Select(Selection::Process(d.row.pid)),
        }
    }
}

fn menu_target(d: &DisplayRow) -> Option<MenuTarget> {
    if d.section.is_some() || d.wsl.is_some() {
        return None;
    }
    if d.absent {
        return Some(MenuTarget::Absent { image: d.row.clone() });
    }
    match &d.child {
        Some(Child::Window(window)) => Some(MenuTarget::Window {
            window: window.clone(),
        }),
        Some(Child::Service(_)) => None,
        Some(Child::Console) | None if d.has_children => Some(MenuTarget::Group {
            leader: d.row.clone(),
        }),
        Some(Child::Console) | None => Some(MenuTarget::Process(d.row.clone())),
    }
}

fn selected_span(rows: &[DisplayRow]) -> Option<Range<usize>> {
    let start = rows.iter().position(|d| d.highlight.is_some())?;
    let len = rows[start..]
        .iter()
        .take_while(|d| d.highlight.is_some())
        .count();
    Some(start..start + len)
}

pub struct ProcessesPage {
    expanded_groups: HashSet<String>,
    expanded_processes: HashSet<u32>,
    collapsed_sections: Option<ReactiveMap<String, bool>>,
    pins: Option<ReactiveMap<String, PinnedProcess>>,
    memory_as_percent: bool,
    by_type: bool,
    by_type_setting: Option<Field<bool>>,
    layout: ColumnLayout,
    groups: RefCell<GroupsCache>,
    kept_place: Cell<Option<(u32, usize)>>,
    selected_group: Option<u32>,
    selected_group_size: Cell<Option<usize>>,
    held: RefCell<Held>,
    icons: Rc<context::IconCache>,
    menu_anchor: Option<(f64, f64)>,
    pending_menu: Option<MenuTarget>,
    menu: Option<OpenMenu>,
    section_order: SectionOrder,
    section_ranks: Option<ReactiveMap<String, u32>>,
    grab: Option<Grab>,
    placement: Cell<Option<Placement>>,
    dropped: Option<SectionId>,
}

impl Default for ProcessesPage {
    fn default() -> Self {
        Self::new(None)
    }
}

fn enabled_keys(map: Option<&ReactiveMap<String, bool>>) -> Vec<String> {
    map.map(|map| {
        map.entries()
            .filter(|(_, on)| *on)
            .map(|(key, _)| key)
            .collect()
    })
    .unwrap_or_default()
}

fn toggle_key(map: Option<&ReactiveMap<String, bool>>, key: String) {
    let Some(map) = map else {
        return;
    };
    let result = if map.get(&key).unwrap_or(false) {
        map.remove(&key).map(|_| ())
    } else {
        map.insert(key.clone(), &true)
    };
    if let Err(err) = result {
        tracing::warn!(%key, ?err, "grouping state write failed");
    }
}

fn toggle_pin(map: Option<&ReactiveMap<String, PinnedProcess>>, name: String, pin: PinnedProcess) {
    let Some(map) = map else {
        return;
    };
    let result = if map.get(&name).is_some() {
        map.remove(&name).map(|_| ())
    } else {
        map.insert(name.clone(), &pin)
    };
    if let Err(err) = result {
        tracing::warn!(%name, ?err, "pin write failed");
    }
}

impl ProcessesPage {
    pub fn new(settings: Option<ProcessesSettingsMaps>) -> Self {
        let memory_as_percent = settings.as_ref().is_some_and(|maps| maps.memory_as_percent.get());
        let (columns, column_order, collapsed_sections, pins, by_type_setting, section_ranks) = match settings {
            Some(maps) => (
                Some(maps.columns),
                Some(maps.column_order),
                Some(maps.collapsed_sections),
                Some(maps.pins),
                Some(maps.group_by_type),
                Some(maps.section_order),
            ),
            None => (None, None, None, None, None, None),
        };
        let section_order = SectionOrder::kept(section_ranks.as_ref());
        Self {
            expanded_groups: HashSet::new(),
            expanded_processes: HashSet::new(),
            collapsed_sections,
            pins,
            memory_as_percent,
            by_type: by_type_setting.as_ref().is_none_or(Field::get),
            by_type_setting,
            layout: ColumnLayout::new(columns, column_order),
            groups: RefCell::new(GroupsCache::empty()),
            kept_place: Cell::new(None),
            selected_group: None,
            selected_group_size: Cell::new(None),
            held: RefCell::new(Held::default()),
            icons: Rc::new(context::IconCache::new()),
            menu_anchor: None,
            pending_menu: None,
            menu: None,
            section_order,
            section_ranks,
            grab: None,
            placement: Cell::new(None),
            dropped: None,
        }
    }

    fn section_gesture(&mut self, gesture: SectionGesture) {
        match gesture {
            SectionGesture::Grab { section, at, offset } => {
                self.dropped = None;
                self.grab = Some(Grab::new(section, at, offset));
            }
            SectionGesture::Move { at } => {
                if let Some(grab) = &mut self.grab {
                    grab.follow(at);
                }
            }
            SectionGesture::Release => {
                let Some(grab) = self.grab.take() else {
                    return;
                };
                if !grab.moved() {
                    return;
                }
                self.dropped = Some(grab.section);
                if let Some(placement) = self.placement.take() {
                    self.move_section(grab.section, placement);
                }
            }
            SectionGesture::Lost => {
                if let Some(grab) = &mut self.grab {
                    grab.let_go();
                }
            }
        }
    }

    fn move_section(&mut self, section: SectionId, placement: Placement) {
        self.section_order = self.section_order.moved(section, placement.before);
        if let Some(ranks) = &self.section_ranks {
            self.section_order.store(ranks);
        }
    }

    pub fn update(&mut self, message: ProcessesMsg) {
        match message {
            ProcessesMsg::Resized(drag) => self.layout.resize(drag),
            ProcessesMsg::Reordered(moved) => self.layout.reorder(moved),
            ProcessesMsg::ToggleGroup(name) => {
                if !self.expanded_groups.remove(&name) {
                    self.expanded_groups.insert(name);
                }
            }
            ProcessesMsg::ToggleProcess(pid) => {
                if !self.expanded_processes.remove(&pid) {
                    self.expanded_processes.insert(pid);
                }
            }
            ProcessesMsg::ToggleSection(section) => {
                let dropped = self.dropped.take() == Some(section);
                if self.menu_anchor.take().is_none() && !dropped {
                    toggle_key(self.collapsed_sections.as_ref(), section.id().to_string())
                }
            }
            ProcessesMsg::Section(gesture) => self.section_gesture(gesture),
            ProcessesMsg::TogglePin(name, pin) => toggle_pin(self.pins.as_ref(), name.to_string(), pin),
            ProcessesMsg::ToggleGroupByType => {
                self.by_type = !self.by_type;
                if let Some(setting) = &self.by_type_setting
                    && let Err(err) = setting.set(self.by_type)
                {
                    tracing::warn!(?err, "group-by-type setting write failed");
                }
            }
            ProcessesMsg::SelectGroup(pid) => self.selected_group = pid,
            ProcessesMsg::MenuAnchor { x, y } => {
                self.menu_anchor = Some((x, y));
                self.menu = self.pending_menu.take().map(|target| OpenMenu { x, y, target });
            }
            ProcessesMsg::MenuFor(target) => {
                if let Some((x, y)) = self.menu_anchor.take() {
                    self.menu = target.map(|target| OpenMenu { x, y, target });
                }
            }
            ProcessesMsg::MenuDismiss => {
                self.menu_anchor = None;
                self.pending_menu = None;
                self.menu = None;
            }
            ProcessesMsg::ColumnMenu => self.pending_menu = Some(MenuTarget::Columns),
            ProcessesMsg::ToggleColumn(column) => self.layout.toggle(column),
        }
    }

    pub fn view(
        &self,
        state: &ProcessesState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesMsg>,
        open_settings: Callback<()>,
        units: Units,
    ) -> View {
        self.selected_group_size.set(None);
        let body = match &state.rows {
            Load::Ready(rows) => {
                self.table(state, rows, dispatch, l10n, palette, forward.clone(), open_settings.clone(), units)
            }
            Load::Failed(err) => text(l10n.processes_failed(err.to_string())).into(),
            _ => loading(),
        };

        let live = state
            .selected
            .and_then(|pid| state.rows().iter().find(|r| r.pid == pid));
        let selected_label = match (live, self.selected_group_size.get()) {
            (Some(row), Some(count)) => l10n.processes_selected_group(row.name.to_string(), count as i64),
            (Some(row), None) => l10n.processes_selected(row.name.to_string(), row.pid as i64),
            (None, _) => state
                .selected
                .and_then(|pid| {
                    self.held
                        .borrow()
                        .row(pid)
                        .map(|r| l10n.processes_selected_exited(r.name.to_string(), r.pid as i64))
                })
                .or_else(|| {
                    let (environment, linux) = state.linux_process(state.selected_linux?)?;
                    let environment = if environment.name.is_empty() {
                        l10n.processes_wsl_namespace(environment.pid_ns.to_string())
                    } else {
                        environment.name.to_string()
                    };
                    Some(l10n.processes_selected_linux(
                        linux.row.name.to_string(),
                        linux.row.pid as i64,
                        environment,
                    ))
                })
                .unwrap_or_default(),
        };

        let tree: Option<Vec<u32>> = self.selected_group_size.get().map(|_| {
            self.held
                .borrow()
                .rows()
                .iter()
                .filter(|row| row.takes_actions() && state.rows().iter().any(|live| live.pid == row.pid))
                .map(|row| row.pid)
                .collect()
        });
        let (end_label, end_enabled) = match &tree {
            Some(pids) => (l10n.processes_end_tree_tasks(), !pids.is_empty()),
            None => (l10n.processes_end_task(), live.is_some_and(ProcessRow::takes_actions)),
        };
        let terminate = dispatch.clone();
        let end = move || match &tree {
            Some(pids) => terminate.emit(TerminateGroup(pids.clone())),
            None => terminate.emit(Terminate),
        };
        let run_new_task = dispatch.clone();
        let header = Grid::new()
            .columns([
                GridLength::Auto,
                GridLength::Star(1.0),
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Auto,
            ])
            .children((
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(space::Header)
                    .grid_column(0)
                    .children((
                        page_title(l10n.processes_title()),
                        text(selected_label)
                            .foreground(palette.secondary_text)
                            .vertical_alignment(VerticalAlignment::Center),
                    )),
                Border::new().grid_column(2).content(command_button(
                    ProcessesMark::RunNewTask,
                    l10n.processes_run_new_task(),
                    Some(icon!(new_task).size(Header::CommandIcon).build_element()),
                    true,
                    move || run_new_task.emit(RunNewTask),
                )),
                Border::new()
                    .grid_column(3)
                    .width(Header::DividerWidth)
                    .height(Header::DividerHeight)
                    .margin(Thickness::xy(space::Control, 0.0))
                    .vertical_alignment(VerticalAlignment::Center)
                    .background(palette.divider_stroke),
                Border::new().mark(SelectionMark::Keeper).grid_column(4).content(command_button(
                    ProcessesMark::EndTask,
                    end_label,
                    Some(icon!(prohibited).size(Header::CommandIcon).build_element()),
                    end_enabled,
                    end,
                )),
                Border::new().grid_column(5).content(icon_button(
                    ProcessesMark::OpenSettings,
                    icon!(more_horizontal).size(Header::CommandIcon).build_element(),
                    move || {
                        let _ = open_settings.call(());
                    },
                )),
            ));

        let body = if matches!(state.agent_state, AgentConnectionState::Connected) {
            body
        } else {
            Grid::new()
                .rows([GridLength::Star(1.0)])
                .children((body, disconnected_overlay(l10n, palette, state.agent_state)))
        };

        let deselect = dispatch.clone();
        let blank = Callback::new(move |()| {
            let _ = forward.call(ProcessesMsg::SelectGroup(None));
            deselect.emit(Deselect);
        });

        let dismiss = dispatch.clone();
        Grid::new()
            .children((
                page_frame(header, body, Self::status(state, l10n, palette), palette, Some(blank)),
                action_failure(state.failure.as_ref(), l10n, move || dismiss.emit(DismissFailure)),
            ))
            .into()
    }

    fn status(state: &ProcessesState, l10n: &L10n, palette: Palette) -> View {
        let of = |wanted: &[ProcessCategory]| {
            state.rows().iter().filter(|row| wanted.contains(&row.category)).count()
        };
        let counts = StatusCounts {
            apps: of(&[ProcessCategory::App]),
            background: of(&[ProcessCategory::BackgroundThirdParty, ProcessCategory::BackgroundMicrosoft]),
            services: of(&[ProcessCategory::WindowsService]),
            kernel: of(&[ProcessCategory::WindowsKernel]),
            linux: state.wsl.iter().map(|environment| environment.processes.len()).sum(),
        };
        status_bar(&counts, l10n, palette)
    }

    fn table(
        &self,
        state: &ProcessesState,
        rows: &[ProcessRow],
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesMsg>,
        open_settings: Callback<()>,
        units: Units,
    ) -> View {
        let collapsed_sections: HashSet<SectionId> =
            enabled_keys(self.collapsed_sections.as_ref())
                .iter()
                .filter_map(|id| SectionId::from_id(id))
                .collect();
        let pins: Pins = self
            .pins
            .as_ref()
            .map(|map| map.entries().map(|(name, pin)| (Arc::from(name), pin)).collect())
            .unwrap_or_default();

        let order = Order {
            column: state.sort_column,
            descending: state.descending,
        };
        let selection = state
            .selected
            .map(|pid| {
                if self.selected_group == Some(pid) {
                    Selection::Group(pid)
                } else {
                    Selection::Process(pid)
                }
            })
            .or(state.selected_linux.map(Selection::Linux));
        let exited_rows = self.held.borrow().exited(selection, rows);
        let kept = self.held.borrow().section(selection);
        let exited: HashSet<u32> = exited_rows.iter().map(|row| row.pid).collect();
        let (display_rows, members) = {
            let mut groups = self.groups.borrow_mut();
            let sections = groups.get(
                rows,
                &exited_rows,
                Grouping {
                    selected: state.selected,
                    kept,
                    order: &order,
                    by_type: self.by_type,
                    pins: &pins,
                    wsl: &state.wsl,
                    section_order: &self.section_order,
                },
            );
            self.kept_place
                .set(keep_group_place(sections, state.selected, self.kept_place.get()));
            self.held.borrow_mut().hold(sections, selection);
            let mut display_rows = flatten_for_display(
                sections,
                &ViewState {
                    groups: &self.expanded_groups,
                    processes: &self.expanded_processes,
                    collapsed_sections: &collapsed_sections,
                    exited: &exited,
                },
            );
            highlight(&mut display_rows, selection);
            let placement = self.grab.and_then(|grab| section_drag::placement(&display_rows, &grab));
            self.placement.set(placement);
            if let Some(grab) = &self.grab {
                section_drag::mark(&mut display_rows, grab, placement);
            }
            let members = match self.menu.as_ref().map(|menu| &menu.target) {
                Some(MenuTarget::Group { leader }) => group_members(sections, leader.pid),
                _ => Vec::new(),
            };
            (display_rows, members)
        };
        if let Some(Selection::Group(pid)) = selection {
            self.selected_group_size.set(
                display_rows
                    .iter()
                    .find(|d| d.has_children && d.section.is_none() && d.row.pid == pid)
                    .map(|d| d.group_size),
            );
        }

        let toggle_forward = forward.clone();
        let process_forward = forward.clone();
        let by_type_forward = forward.clone();
        let gesture_forward = forward.clone();
        let group_by_type = GroupByType {
            on: self.by_type,
            toggle: Callback::new(move |()| {
                let _ = by_type_forward.call(ProcessesMsg::ToggleGroupByType);
            }),
        };
        let actions = NameCellActions {
            icons: self.icons.clone(),
            toggle_group: Callback::new(move |name: String| {
                let _ = toggle_forward.call(ProcessesMsg::ToggleGroup(name));
            }),
            toggle_process: Callback::new(move |pid: u32| {
                let _ = process_forward.call(ProcessesMsg::ToggleProcess(pid));
            }),
            section_gesture: Callback::new(move |gesture: SectionGesture| {
                let _ = gesture_forward.call(ProcessesMsg::Section(gesture));
            }),
        };

        let span = selected_span(&display_rows);
        let presses: Vec<Press> = display_rows.iter().map(Press::of).collect();
        let targets: Vec<Option<MenuTarget>> = display_rows.iter().map(menu_target).collect();
        let select = dispatch.clone();
        let press_forward = forward.clone();
        let on_press = Callback::new(move |at: Option<usize>| {
            if at.is_none() {
                let _ = press_forward.call(ProcessesMsg::SelectGroup(None));
                select.emit(Deselect);
            }
            match at.and_then(|at| presses.get(at)) {
                Some(Press::Toggle(category)) => {
                    let _ = press_forward.call(ProcessesMsg::ToggleSection(*category));
                }
                Some(Press::Expand(key)) => {
                    let _ = press_forward.call(ProcessesMsg::ToggleGroup(key.clone()));
                }
                Some(Press::Select(selection)) => {
                    let group = match *selection {
                        Selection::Group(pid) => Some(pid),
                        Selection::Process(_) | Selection::Linux(_) => None,
                    };
                    let _ = press_forward.call(ProcessesMsg::SelectGroup(group));
                    match *selection {
                        Selection::Group(pid) | Selection::Process(pid) => select.emit(Select(pid)),
                        Selection::Linux(global_pid) => select.emit(SelectLinux(global_pid)),
                    }
                }
                Some(Press::Nothing) | None => {}
            }
            let target = at.and_then(|at| targets.get(at)).cloned().flatten();
            let _ = press_forward.call(ProcessesMsg::MenuFor(target));
        });

        let pointer_forward = forward.clone();
        let on_pointer_pressed = Callback::new(move |pointer: PointerEventInfo| {
            let message = if pointer.is_right_button_pressed {
                ProcessesMsg::MenuAnchor {
                    x: pointer.x,
                    y: pointer.y,
                }
            } else {
                ProcessesMsg::MenuDismiss
            };
            let _ = pointer_forward.call(message);
        });

        let menu = self.menu.as_ref().map(|menu| {
            let command_dispatch = dispatch.clone();
            let command_forward = forward.clone();
            let dismiss_forward = forward.clone();
            let pin = menu.target.pin();
            let pinned = pin.as_ref().is_some_and(|(name, _)| pins.contains_key(name));
            let image = menu
                .target
                .image()
                .map(|image| (image.exe_path.to_string(), image.name.to_string()));
            let group: Vec<u32> = members.iter().map(|row| row.pid).collect();
            context_menu(menu, MenuInputs {
                l10n,
                palette,
                pinned,
                members: &members,
                columns: self.layout.columns().iter().map(|c| (c.column, c.visible)).collect(),
                on_command: Callback::new(move |command: MenuCommand| {
                    match command {
                        MenuCommand::TogglePin => {
                            if let Some((name, pin)) = pin.clone() {
                                let _ = command_forward.call(ProcessesMsg::TogglePin(name, pin));
                            }
                        }
                        MenuCommand::EndTask => command_dispatch.emit(Terminate),
                        MenuCommand::EndGroup => command_dispatch.emit(TerminateGroup(group.clone())),
                        MenuCommand::Process(command) => match image.clone() {
                            Some((exe_path, name)) => command_dispatch.emit(RunImageCommand { command, exe_path, name }),
                            None => command_dispatch.emit(RunProcessCommand(command)),
                        },
                        MenuCommand::Window { handle, command } => {
                            command_dispatch.emit(RunWindowCommand { handle, command })
                        }
                        MenuCommand::ToggleColumn(column) => {
                            let _ = command_forward.call(ProcessesMsg::ToggleColumn(column));
                        }
                        MenuCommand::OpenSettings => {
                            let _ = open_settings.call(());
                        }
                    }
                    let _ = command_forward.call(ProcessesMsg::MenuDismiss);
                }),
                on_dismiss: Callback::new(move |()| {
                    let _ = dismiss_forward.call(ProcessesMsg::MenuDismiss);
                }),
            })
        });

        let header_forward = forward.clone();
        let columns = build_columns(ColumnInputs {
            layout: &self.layout,
            machine: state.machine_summary().cloned(),
            rows,
            memory_as_percent: self.memory_as_percent,
            units,
            actions,
            group_by_type,
            sort_column: state.sort_column,
            descending: state.descending,
            palette,
            l10n,
            header_menu: Callback::new(move |()| {
                let _ = header_forward.call(ProcessesMsg::ColumnMenu);
            }),
        });

        let sort = dispatch.clone();
        let reorder_forward = forward.clone();
        let resize_forward = forward;

        let table = table(display_rows, columns)
            .look(Look {
                row_height: size::ProcessRow,
                hovered: palette.row_hovered,
                selected: palette.row_selected,
                inset: (space::Compact, space::Hairline),
                radius: radius::Control,
                separator: palette.divider_stroke,
                rule: palette.divider_stroke,
            })
            .corner_radius(radius::Card)
            .selection(span.as_ref().map(|span| span.start), on_press);
        let table = match span {
            Some(span) => table.selected_span(span),
            None => table,
        };

        let table = table
            .widths(self.layout.widths())
            .on_resize(move |drag: Resized| {
                let _ = resize_forward.call(ProcessesMsg::Resized(drag));
            })
            .order(self.layout.order())
            .on_reorder(move |moved: Reordered| {
                let _ = reorder_forward.call(ProcessesMsg::Reordered(moved));
            })
            .sort(
                SortState {
                    field_id: Some(state.sort_column),
                    descending: state.descending,
                },
                move |column: ProcessColumn| sort.emit(Sort(column)),
            )
            .sort_indicator(|_| View::empty())
            .build();

        Grid::new()
            .mark(SelectionMark::Keeper)
            .children((
                Border::new()
                    .on_pointer_pressed(on_pointer_pressed)
                    .content(table),
                menu.unwrap_or_else(View::empty),
            ))
            .into()
    }
}
