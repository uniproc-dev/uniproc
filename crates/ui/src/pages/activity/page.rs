use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityState, ActivityView, ClearRange, NewOnly, PickBucket, Search, ShowBursts, ShowCame, ShowSpan, ShowWent,
    Span,
};
use app_contracts::features::agents::ProcessInstance;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Border, Callback, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength, HorizontalAlignment,
    LayoutControl, Orientation, ScrollBarVisibility, ScrollViewer, StackPanel, TextBox, Thickness, ToggleButton,
    VerticalAlignment, View,
};

use super::components::histogram::histogram;
use super::components::rows::{rows, Rows};
use super::marks::ActivityMark;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::button::command_button;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::text::{caption, text};

struct SearchBox;

#[expect(non_upper_case_globals)]
impl SearchBox {
    const Width: f64 = 260.0;
}

pub enum ActivityPageMsg {
    Toggle(ProcessInstance),
}

#[derive(Default)]
pub struct ActivityPage {
    expanded: Rc<HashSet<ProcessInstance>>,
}

fn chip(mark: impl Mark, label: String, checked: bool, on_change: impl Fn(bool) + 'static) -> View {
    ToggleButton::new()
        .mark(mark)
        .is_checked(checked)
        .on_is_checked_changed(on_change)
        .content(text(label))
}

fn span_label(span: Span, l10n: &L10n) -> String {
    match span {
        Span::Quarter => l10n.activity_span_quarter(),
        Span::Hour => l10n.activity_span_hour(),
        Span::Connected => l10n.activity_span_connected(),
    }
}

impl ActivityPage {
    pub fn update(&mut self, message: ActivityPageMsg) {
        match message {
            ActivityPageMsg::Toggle(key) => {
                let expanded = Rc::make_mut(&mut self.expanded);
                if !expanded.remove(&key) {
                    expanded.insert(key);
                }
            }
        }
    }

    fn header(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n) -> View {
        let filter = &state.filter;
        let (came, went, new_only, bursts) = (dispatch.clone(), dispatch.clone(), dispatch.clone(), dispatch.clone());
        let spans: Vec<(String, View)> = [Span::Quarter, Span::Hour, Span::Connected]
            .into_iter()
            .map(|span| {
                let dispatch = dispatch.clone();
                (
                    span.name().to_string(),
                    chip(span, span_label(span, l10n), state.span == span, move |_| {
                        dispatch.emit(ShowSpan(span))
                    }),
                )
            })
            .collect();

        Grid::new()
            .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
            .children((
                StackPanel::new()
                    .grid_column(0)
                    .orientation(Orientation::Horizontal)
                    .spacing(space::Control)
                    .children((
                        page_title(l10n.activity_title()),
                        chip(ActivityMark::Came, l10n.activity_came(), filter.came, move |on| {
                            came.emit(ShowCame(on))
                        }),
                        chip(ActivityMark::Went, l10n.activity_went(), filter.went, move |on| {
                            went.emit(ShowWent(on))
                        }),
                        chip(ActivityMark::NewOnly, l10n.activity_new_only(), filter.new_only, move |on| {
                            new_only.emit(NewOnly(on))
                        }),
                        chip(ActivityMark::Bursts, l10n.activity_bursts(), filter.bursts, move |on| {
                            bursts.emit(ShowBursts(on))
                        }),
                    )),
                StackPanel::new()
                    .grid_column(2)
                    .orientation(Orientation::Horizontal)
                    .spacing(space::Control)
                    .children((View::keyed_fragment(spans),)),
            ))
    }

    fn range_bar(view: &ActivityView, state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let searched = dispatch.clone();
        let cleared = dispatch.clone();
        let since = view
            .history_since
            .map(|at| l10n.activity_history_since(format::clock(at)))
            .unwrap_or_default();
        Grid::new()
            .columns([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
            .margin(Thickness::xy(space::Cell, space::Control))
            .children((
                caption(since)
                    .grid_column(0)
                    .foreground(palette.secondary_text)
                    .vertical_alignment(VerticalAlignment::Center),
                TextBox::new()
                    .mark(ActivityMark::Search)
                    .grid_column(1)
                    .width(SearchBox::Width)
                    .text(state.filter.text.clone())
                    .placeholder_text(l10n.activity_search())
                    .on_text_changed(move |text: String| searched.emit(Search(text))),
                Border::new()
                    .grid_column(2)
                    .margin(Thickness::new(space::Control, 0.0, 0.0, 0.0))
                    .content(command_button(
                        ActivityMark::ClearRange,
                        l10n.activity_clear_range(),
                        None,
                        view.histogram.picked.is_some(),
                        move || cleared.emit(ClearRange),
                    )),
            ))
    }

    pub fn view(
        &self,
        state: &ActivityState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ActivityPageMsg>,
    ) -> View {
        let header = Self::header(state, dispatch, l10n);
        let Load::Ready(view) = &state.view else {
            return page_frame(header, loading(), status_text(l10n.activity_loading(), palette), palette);
        };

        let picked = dispatch.clone();
        let list: View = if view.rows.is_empty() {
            text(l10n.activity_empty())
                .mark(ActivityMark::Empty)
                .horizontal_alignment(HorizontalAlignment::Center)
                .margin(Thickness::uniform(space::Section))
                .into()
        } else {
            ScrollViewer::new()
                .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                .content(rows(Rows {
                    rows: Rc::from(view.rows.as_slice()),
                    expanded: self.expanded.clone(),
                    l10n: l10n.clone(),
                    palette,
                    on_toggle: Callback::new(move |key: ProcessInstance| {
                        let _ = forward.call(ActivityPageMsg::Toggle(key));
                    }),
                }))
        };

        let body = Grid::new()
            .rows([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
            .children((
                Border::new()
                    .grid_row(0)
                    .margin(Thickness::new(space::Cell, space::Card, space::Cell, 0.0))
                    .content(histogram(
                        &view.histogram,
                        l10n,
                        palette,
                        Callback::new(move |at: usize| picked.emit(PickBucket(at))),
                    )),
                Border::new()
                    .grid_row(1)
                    .content(Self::range_bar(view, state, dispatch, l10n, palette)),
                Border::new().grid_row(2).content(list),
            ));

        let status = l10n.activity_range(
            format::clock(view.from),
            format::clock(view.to),
            view.came as i64,
            view.went as i64,
        );
        page_frame(header, body, status_text(status, palette), palette)
    }
}
