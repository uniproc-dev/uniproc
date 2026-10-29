use std::sync::Arc;
use std::cell::RefCell;

use app_contracts::features::services::{
    Command, Deselect, DismissFailure, OpenServices, RunNewTask, Select, ServiceActionKind, ServiceColumn, ServiceRow, ServicesState, Sort,
};
use guinea::prelude::{Dispatch, Load};
use crate::widgets::table::{table, Look, Reordered, Resized, SortState};
use guicons::icon;
use windows_reactor::{Border, Callback, Grid, GridLength, Orientation, PointerEventInfo, StackPanel, View};
use guinea::winui::MarkExt;

use super::components::columns::{build_columns, Heading};
use super::components::context_menu::context_menu;
use super::components::column_layout::{ColumnLayout, ServicesSettingsMaps};
use super::marks::ServicesMark;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::action_failure::action_failure;
use crate::widgets::button::{command_button, command_divider, icon_button};
use crate::widgets::selection::{keep_place, SelectionMark};
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::text::text;

pub enum ServicesMsg {
    Resized(Resized),
    Reordered(Reordered),
    ColumnMenu,
    MenuAnchor { x: f64, y: f64 },
    MenuDismiss,
    ToggleColumn(ServiceColumn),
}

#[derive(Default)]
pub struct ServicesPage {
    layout: ColumnLayout,
    menu: Option<(f64, f64)>,
    pending_menu: bool,
    kept_place: RefCell<Option<(Arc<str>, usize)>>,
}

fn command(dispatch: &Dispatch, kind: ServiceActionKind) -> impl Fn() + 'static {
    let dispatch = dispatch.clone();
    move || dispatch.emit(Command(kind))
}

impl ServicesPage {
    pub fn new(settings: ServicesSettingsMaps) -> Self {
        Self { layout: ColumnLayout::new(Some(settings)), ..Default::default() }
    }

    pub fn update(&mut self, message: ServicesMsg) {
        match message {
            ServicesMsg::Resized(drag) => self.layout.resize(drag),
            ServicesMsg::Reordered(moved) => self.layout.reorder(moved),
            ServicesMsg::ColumnMenu => {
                self.pending_menu = true;
            }
            ServicesMsg::MenuAnchor { x, y } => {
                if self.pending_menu { self.menu = Some((x, y)); self.pending_menu = false; }
                else { self.menu = None; }
            }
            ServicesMsg::MenuDismiss => { self.menu = None; self.pending_menu = false; }
            ServicesMsg::ToggleColumn(column) => self.layout.show(column, !self.layout.config(column).visible),
        }
    }

    pub fn view(
        &self,
        state: &ServicesState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ServicesMsg>,
        open_settings: Callback<()>,
    ) -> View {
        let selected = state.rows().iter().find(|row| Some(row.name.as_ref()) == state.selected.as_deref());
        let can_start = selected.is_some_and(ServiceRow::can_start);
        let can_stop = selected.is_some_and(ServiceRow::can_stop);

        let run = dispatch.clone();
        let open = dispatch.clone();
        let settings = open_settings.clone();
        let commands = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .grid_column(2)
            .children((
                command_button(
                    ServicesMark::RunNewTask,
                    l10n.services_run_new_task(),
                    Some(icon!(new_task).size(size::CommandIcon).build_element()),
                    true,
                    move || run.emit(RunNewTask),
                ),
                command_divider(palette),
                Border::new().mark(SelectionMark::Keeper).content(command_button(
                    ServicesMark::Start,
                    l10n.services_start(),
                    Some(icon!(play, color = if can_start { palette.success } else { palette.disabled_text }).size(size::CommandIcon).build_element()),
                    can_start,
                    command(dispatch, ServiceActionKind::Start),
                )),
                Border::new().mark(SelectionMark::Keeper).content(command_button(
                    ServicesMark::Stop,
                    l10n.services_stop(),
                    Some(icon!(stop, color = if can_stop { palette.critical } else { palette.disabled_text }).size(size::CommandIcon).build_element()),
                    can_stop,
                    command(dispatch, ServiceActionKind::Stop),
                )),
                Border::new().mark(SelectionMark::Keeper).content(command_button(
                    ServicesMark::Restart,
                    l10n.services_restart(),
                    Some(icon!(service_restart, color = if can_stop { palette.caution } else { palette.disabled_text }).size(size::CommandIcon).build_element()),
                    can_stop,
                    command(dispatch, ServiceActionKind::Restart),
                )),
                command_divider(palette),
                command_button(
                    ServicesMark::OpenServices,
                    l10n.services_open_services(),
                    Some(icon!(open_services).size(size::CommandIcon).build_element()),
                    true,
                    move || open.emit(OpenServices),
                ),
                icon_button(ServicesMark::OpenSettings,
                    icon!(more_horizontal).size(size::CommandIcon).build_element(),
                    move || settings.call(())),
            ));
        let header = Grid::new()
            .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
            .children((page_title(l10n.services_title()), commands));

        let body = match &state.rows {
            Load::Ready(rows) => {
                let mut rows: Vec<ServiceRow> = rows.to_vec();
                let previous = self.kept_place.borrow_mut().take();
                let pinned = state.selected.as_deref().and_then(|name| {
                    keep_place(&mut rows, Arc::from(name), |row| row.name.as_ref() == name, previous)
                });
                *self.kept_place.borrow_mut() = pinned;
                let selected = state
                    .selected
                    .as_deref()
                    .and_then(|name| rows.iter().position(|r| &*r.name == name));
                let names: Vec<Arc<str>> = rows.iter().map(|r| r.name.clone()).collect();
                let select = dispatch.clone();
                let sort = dispatch.clone();
                let resized = forward.clone();
                let reordered = forward.clone();
                let menu = forward.clone();

                let table = table(rows, build_columns(l10n, palette, &self.layout, state.selected.as_deref(), Heading {
                    column: state.sort_column, descending: state.descending,
                    menu: Callback::new(move |()| menu.call(ServicesMsg::ColumnMenu)),
                }))
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
                .widths(self.layout.widths())
                .order(&self.layout.order())
                .on_reorder(move |moved: Reordered| reordered.call(ServicesMsg::Reordered(moved)))
                .on_resize(move |drag: Resized| {
                    resized.call(ServicesMsg::Resized(drag));
                })
                .sort(
                    SortState {
                        field_id: Some(state.sort_column),
                        descending: state.descending,
                    },
                    move |column: ServiceColumn| sort.emit(Sort(column)),
                )
                .sort_indicator(|_| None)
                .selection(selected, move |at: Option<usize>| {
                    if let Some(name) = at.and_then(|at| names.get(at)) {
                        select.emit(Select(name.to_string()));
                    } else {
                        select.emit(Deselect);
                    }
                })
                .build();
                let pressed = forward.clone();
                let mut layers: Vec<View> = vec![Border::new().on_pointer_pressed(Callback::new(move |pointer: PointerEventInfo| {
                    if pointer.is_right_button_pressed {
                        pressed.call(ServicesMsg::MenuAnchor { x: pointer.x, y: pointer.y });
                    } else { pressed.call(ServicesMsg::MenuDismiss); }
                })).content(table).into()];
                if let Some(anchor) = self.menu {
                    let selected = forward.clone();
                    let dismiss = forward.clone();
                    layers.push(context_menu(anchor, &self.layout, l10n, palette,
                        Callback::new(move |column| {
                            if let Some(column) = column { selected.call(ServicesMsg::ToggleColumn(column)); }
                            else { open_settings.call(()); }
                            selected.call(ServicesMsg::MenuDismiss);
                        }), Callback::new(move |()| dismiss.call(ServicesMsg::MenuDismiss))));
                }
                Grid::new().mark(SelectionMark::Keeper).children(layers).into()
            }
            Load::Failed(err) => text(l10n.services_failed(err.to_string())).into(),
            _ => loading(),
        };

        let dismiss = dispatch.clone();
        Grid::new()
            .children((
                page_frame(
                    header,
                    body,
                    status_text(l10n.services_status(state.total() as i64), palette),
                    palette,
                ),
                action_failure(state.failure.as_ref(), l10n, move || dismiss.emit(DismissFailure)),
            ))
            .into()
    }
}
