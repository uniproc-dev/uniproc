use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use app_contracts::features::services::{
    Command, DismissFailure, Select, ServiceActionKind, ServiceColumn, ServiceRow, ServicesState, Sort,
};
use guinea::prelude::{Dispatch, Load};
use crate::widgets::table::{table, ColumnWidths, Resized, SortState};
use windows_reactor::{Callback, Grid, Orientation, StackPanel, View};

use super::components::columns::build_columns;
use super::marks::ServicesMark;
use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::action_failure::action_failure;
use crate::widgets::button::command_button;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::text::text;

pub enum ServicesMsg {
    Resized(Resized),
}

#[derive(Default)]
pub struct ServicesPage {
    widths: ColumnWidths,
}

fn command(dispatch: &Dispatch, kind: ServiceActionKind) -> impl Fn() + 'static {
    let dispatch = dispatch.clone();
    move || dispatch.emit(Command(kind))
}

impl ServicesPage {
    pub fn update(&mut self, message: ServicesMsg) {
        match message {
            ServicesMsg::Resized(drag) => self.widths.apply(drag),
        }
    }

    pub fn view(
        &self,
        state: &ServicesState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ServicesMsg>,
    ) -> View {
        let has_selection = state.selected.is_some();

        let header = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Header)
            .children((
                page_title(l10n.services_title()),
                command_button(
                    ServicesMark::Start,
                    l10n.services_start(),
                    None,
                    has_selection,
                    command(dispatch, ServiceActionKind::Start),
                ),
                command_button(
                    ServicesMark::Stop,
                    l10n.services_stop(),
                    None,
                    has_selection,
                    command(dispatch, ServiceActionKind::Stop),
                ),
                command_button(
                    ServicesMark::Restart,
                    l10n.services_restart(),
                    None,
                    has_selection,
                    command(dispatch, ServiceActionKind::Restart),
                ),
            ));

        let body = match &state.rows {
            Load::Ready(rows) => {
                let rows: Vec<ServiceRow> = rows.to_vec();
                let selected = state
                    .selected
                    .as_deref()
                    .and_then(|name| rows.iter().position(|r| &*r.name == name));
                let names: Vec<Arc<str>> = rows.iter().map(|r| r.name.clone()).collect();
                let select = dispatch.clone();
                let sort = dispatch.clone();

                table(rows, build_columns(l10n))
                .key(|row: &ServiceRow| {
                    let mut hasher = DefaultHasher::new();
                    row.name.hash(&mut hasher);
                    hasher.finish()
                })
                .palette(palette)
                .widths(&self.widths)
                .on_resize(move |drag: Resized| {
                    forward.call(ServicesMsg::Resized(drag));
                })
                .sort(
                    SortState {
                        field_id: Some(state.sort_column),
                        descending: state.descending,
                    },
                    move |column: ServiceColumn| sort.emit(Sort(column)),
                )
                .selection(selected, move |at: Option<usize>| {
                    if let Some(name) = at.and_then(|at| names.get(at)) {
                        select.emit(Select(name.to_string()));
                    }
                })
                .build()
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
