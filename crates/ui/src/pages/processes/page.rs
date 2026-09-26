use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;

use amethystate::ReactiveMap;
use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::processes::{
    ColumnConfig, ProcessCategory, ProcessColumn, ProcessRow, ProcessesState, Select, Sort,
    Terminate,
};
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea_widgets::table::{table, Look, Resized, SortState};
use windows_reactor::{
    Callback, ChildrenControl, Grid, GridLength, LayoutControl, Orientation, StackPanel,
    VerticalAlignment, View,
};

use super::components::column_layout::ColumnLayout;
use super::components::columns::{build_columns, ColumnInputs, NameCellActions};
use super::components::grouping::{
    flatten_for_display, highlight, pin_group, DisplayRow, GroupsCache, Held, Order, Selection,
    ViewState,
};
use super::components::overlay::disconnected_overlay;
use super::marks::ProcessesMark;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::page::{command_button, loading, page_frame, page_title};
use crate::widgets::text::text;

pub struct ProcessesSettingsMaps {
    pub columns: ReactiveMap<String, ColumnConfig>,
    pub collapsed_sections: ReactiveMap<String, bool>,
}

pub enum ProcessesMsg {
    Resized(Resized),
    ToggleGroup(String),
    ToggleProcess(u32),
    ToggleSection(ProcessCategory),
    SelectGroup(Option<u32>),
}

enum Press {
    Toggle(ProcessCategory),
    Select(Selection),
}

impl Press {
    fn of(d: &DisplayRow) -> Self {
        match &d.section {
            Some(section) => Self::Toggle(section.category),
            None if d.has_children => Self::Select(Selection::Group(d.row.pid)),
            None => Self::Select(Selection::Process(d.row.pid)),
        }
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
    layout: ColumnLayout,
    groups: RefCell<GroupsCache>,
    pinned: Cell<Option<(u32, usize)>>,
    selected_group: Option<u32>,
    held: RefCell<Held>,
    icons: Rc<context::IconCache>,
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
        let (columns, collapsed_sections) = match settings {
            Some(maps) => (Some(maps.columns), Some(maps.collapsed_sections)),
            None => (None, None),
        };
        Self {
            expanded_groups: HashSet::new(),
            expanded_processes: HashSet::new(),
            collapsed_sections,
            layout: ColumnLayout::new(columns),
            groups: RefCell::new(GroupsCache::empty()),
            pinned: Cell::new(None),
            selected_group: None,
            held: RefCell::new(Held::default()),
            icons: Rc::new(context::IconCache::new()),
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
            ProcessesMsg::ToggleSection(category) => {
                toggle_key(self.collapsed_sections.as_ref(), category.id().to_string())
            }
            ProcessesMsg::SelectGroup(pid) => self.selected_group = pid,
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
        let live = state
            .selected
            .and_then(|pid| state.rows().iter().find(|r| r.pid == pid));
        let selected_name = match live {
            Some(row) => row.name.to_string(),
            None => state
                .selected
                .and_then(|pid| {
                    self.held
                        .borrow()
                        .row(pid)
                        .map(|r| format!("{} — {}", r.name, l10n.processes_exited()))
                })
                .unwrap_or_default(),
        };

        let terminate = dispatch.clone();
        let header = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Header)
            .children((
                page_title(l10n.processes_title()),
                text(selected_name)
                    .foreground(palette.secondary_text)
                    .vertical_alignment(VerticalAlignment::Center),
                command_button(
                    ProcessesMark::EndTask,
                    l10n.processes_end_task(),
                    Some(icon!(prohibited).size(size::CommandIcon).build_element()),
                    live.is_some(),
                    move || terminate.emit(Terminate),
                ),
            ));

        let body = match &state.rows {
            Load::Ready(rows) => self.table(state, rows, dispatch, l10n, palette, forward),
            Load::Failed(err) => text(l10n.processes_failed(err.to_string())).into(),
            _ => loading(),
        };

        let body = if matches!(state.agent_state, AgentConnectionState::Connected) {
            body
        } else {
            Grid::new()
                .rows([GridLength::Star(1.0)])
                .children((body, disconnected_overlay(l10n, palette, state.agent_state)))
        };

        page_frame(
            header,
            body,
            l10n.processes_status(state.total() as i64),
            palette,
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
        let collapsed_sections: HashSet<ProcessCategory> =
            enabled_keys(self.collapsed_sections.as_ref())
                .iter()
                .filter_map(|id| ProcessCategory::from_id(id))
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
            let sections = groups.get(rows, &exited_rows, state.selected, kept, &order);
            self.pinned.set(pin_group(sections, state.selected, self.pinned.get()));
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

        let toggle_forward = forward.clone();
        let process_forward = forward.clone();
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
        let select = dispatch.clone();
        let press_forward = forward.clone();
        let on_press = Callback::new(move |at: Option<usize>| {
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
        });

        let columns = build_columns(ColumnInputs {
            layout: &self.layout,
            machine: state.machine_summary().cloned(),
            rows,
            actions,
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
            .selection(span.as_ref().map(|span| span.start), on_press);
        let table = match span {
            Some(span) => table.selected_span(span),
            None => table,
        };

        table
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
            .build()
    }
}
