use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use amethystate::{Field, ReactiveMap};
use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::processes::{
    ColumnConfig, Deselect, ProcessColumn, ProcessRow, ProcessesState, RunProcessCommand,
    RunWindowCommand, Select, Sort, Terminate,
};
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use guinea_widgets::table::{table, Look, Resized, SortState};
use windows_reactor::{
    Border, Callback, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, LayoutControl,
    PointerEventInfo, Orientation, StackPanel,
    VerticalAlignment, View,
};

use super::components::column_layout::ColumnLayout;
use super::components::columns::{build_columns, ColumnInputs, GroupByType, NameCellActions};
use super::components::context_menu::{context_menu, MenuCommand, MenuInputs, MenuTarget, OpenMenu};
use super::components::grouping::{
    flatten_for_display, highlight, keep_group_place, Child, DisplayRow, Grouping, GroupsCache,
    Held, Order, SectionId, Selection, ViewState,
};
use super::components::overlay::disconnected_overlay;
use super::marks::ProcessesMark;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::page::{command_button, loading, page_frame, page_title};
use crate::widgets::selection::SelectionMark;
use crate::widgets::text::text;

pub struct ProcessesSettingsMaps {
    pub columns: ReactiveMap<String, ColumnConfig>,
    pub collapsed_sections: ReactiveMap<String, bool>,
    pub group_by_type: Field<bool>,
    pub pinned: ReactiveMap<String, bool>,
}

pub enum ProcessesMsg {
    Resized(Resized),
    ToggleGroup(String),
    ToggleProcess(u32),
    ToggleSection(SectionId),
    TogglePin(Arc<str>),
    ToggleGroupByType,
    SelectGroup(Option<u32>),
    MenuAnchor { x: f64, y: f64 },
    MenuFor(Option<MenuTarget>),
    MenuDismiss,
}

enum Press {
    Toggle(SectionId),
    Select(Selection),
}

impl Press {
    fn of(d: &DisplayRow) -> Self {
        match &d.section {
            Some(section) => Self::Toggle(section.id),
            None if d.has_children => Self::Select(Selection::Group(d.row.pid)),
            None => Self::Select(Selection::Process(d.row.pid)),
        }
    }
}

fn menu_target(d: &DisplayRow) -> Option<MenuTarget> {
    if d.section.is_some() {
        return None;
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
    pins: Option<ReactiveMap<String, bool>>,
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
    menu: Option<OpenMenu>,
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

impl ProcessesPage {
    pub fn new(settings: Option<ProcessesSettingsMaps>) -> Self {
        let (columns, collapsed_sections, pins, by_type_setting) = match settings {
            Some(maps) => (
                Some(maps.columns),
                Some(maps.collapsed_sections),
                Some(maps.pinned),
                Some(maps.group_by_type),
            ),
            None => (None, None, None, None),
        };
        Self {
            expanded_groups: HashSet::new(),
            expanded_processes: HashSet::new(),
            collapsed_sections,
            pins,
            by_type: by_type_setting.as_ref().is_none_or(Field::get),
            by_type_setting,
            layout: ColumnLayout::new(columns),
            groups: RefCell::new(GroupsCache::empty()),
            kept_place: Cell::new(None),
            selected_group: None,
            selected_group_size: Cell::new(None),
            held: RefCell::new(Held::default()),
            icons: Rc::new(context::IconCache::new()),
            menu_anchor: None,
            menu: None,
        }
    }

    pub fn update(&mut self, message: ProcessesMsg) {
        match message {
            ProcessesMsg::Resized(drag) => self.layout.resize(drag),
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
                if self.menu_anchor.take().is_none() {
                    toggle_key(self.collapsed_sections.as_ref(), section.id().to_string())
                }
            }
            ProcessesMsg::TogglePin(name) => toggle_key(self.pins.as_ref(), name.to_string()),
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
                self.menu = None;
            }
            ProcessesMsg::MenuFor(target) => {
                if let Some((x, y)) = self.menu_anchor.take() {
                    self.menu = target.map(|target| OpenMenu { x, y, target });
                }
            }
            ProcessesMsg::MenuDismiss => {
                self.menu_anchor = None;
                self.menu = None;
            }
        }
    }

    pub fn view(
        &self,
        state: &ProcessesState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesMsg>,
    ) -> View {
        self.selected_group_size.set(None);
        let body = match &state.rows {
            Load::Ready(rows) => self.table(state, rows, dispatch, l10n, palette, forward.clone()),
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
                .unwrap_or_default(),
        };

        let terminate = dispatch.clone();
        let header = Grid::new()
            .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
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
                Border::new().mark(SelectionMark::Keeper).grid_column(2).content(command_button(
                    ProcessesMark::EndTask,
                    l10n.processes_end_task(),
                    Some(icon!(prohibited).size(size::CommandIcon).build_element()),
                    live.is_some(),
                    move || terminate.emit(Terminate),
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

        page_frame(
            header,
            body,
            l10n.processes_status(state.total() as i64),
            palette,
            Some(blank),
        )
    }

    fn table(
        &self,
        state: &ProcessesState,
        rows: &[ProcessRow],
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesMsg>,
    ) -> View {
        let collapsed_sections: HashSet<SectionId> =
            enabled_keys(self.collapsed_sections.as_ref())
                .iter()
                .filter_map(|id| SectionId::from_id(id))
                .collect();
        let pins: HashSet<Arc<str>> = enabled_keys(self.pins.as_ref())
            .into_iter()
            .map(Arc::from)
            .collect();

        let order = Order {
            column: state.sort_column,
            descending: state.descending,
        };
        let selection = state.selected.map(|pid| {
            if self.selected_group == Some(pid) {
                Selection::Group(pid)
            } else {
                Selection::Process(pid)
            }
        });
        let exited_rows = self.held.borrow().exited(selection, rows);
        let kept = self.held.borrow().section(selection);
        let exited: HashSet<u32> = exited_rows.iter().map(|row| row.pid).collect();
        let display_rows = {
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
            display_rows
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
                Some(Press::Select(selection)) => {
                    let (pid, group) = match *selection {
                        Selection::Group(pid) => (pid, Some(pid)),
                        Selection::Process(pid) => (pid, None),
                    };
                    let _ = press_forward.call(ProcessesMsg::SelectGroup(group));
                    select.emit(Select(pid));
                }
                None => {}
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
            let pin_key = menu.target.pin_key();
            let pinned = pin_key.as_ref().is_some_and(|key| pins.contains(key));
            context_menu(menu, MenuInputs {
                l10n,
                palette,
                pinned,
                on_command: Callback::new(move |command: MenuCommand| {
                    match command {
                        MenuCommand::TogglePin => {
                            if let Some(key) = pin_key.clone() {
                                let _ = command_forward.call(ProcessesMsg::TogglePin(key));
                            }
                        }
                        MenuCommand::EndTask => command_dispatch.emit(Terminate),
                        MenuCommand::Process(command) => {
                            command_dispatch.emit(RunProcessCommand(command))
                        }
                        MenuCommand::Window { handle, command } => {
                            command_dispatch.emit(RunWindowCommand { handle, command })
                        }
                    }
                    let _ = command_forward.call(ProcessesMsg::MenuDismiss);
                }),
                on_dismiss: Callback::new(move |()| {
                    let _ = dismiss_forward.call(ProcessesMsg::MenuDismiss);
                }),
            })
        });

        let columns = build_columns(ColumnInputs {
            layout: &self.layout,
            machine: state.machine_summary().cloned(),
            rows,
            actions,
            group_by_type,
            sort_column: state.sort_column,
            descending: state.descending,
            palette,
            l10n,
        });

        let sort = dispatch.clone();
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
