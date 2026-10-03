use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityState, ActivityView, Area, ClearArea, Hover, NewOnly, PickArea, Search, ShowBursts, ShowCame, ShowSpan,
    ShowWent, Span,
};
use app_contracts::features::agents::ProcessInstance;
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, CheckBox, ChildrenControl, ContentControl, Flyout, FlyoutExt,
    FlyoutPlacement, Grid, GridChildExt, GridLength, HorizontalAlignment, LayoutControl, Orientation, RadioButton,
    ScrollBarVisibility, ScrollViewer, StackPanel, TextBox, Thickness, VerticalAlignment, View,
};

use super::components::card::card;
use super::components::lasted::lasted;
use super::components::rows::{rows, Rows};
use super::components::scatter::{labels, Plotted, ScatterPlot};
use super::marks::ActivityMark;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::button::command_button;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::separator;
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

fn shown(mark: impl Mark, label: String, checked: bool, on_change: impl Fn(bool) + 'static) -> View {
    CheckBox::new()
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

    fn menu(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let filter = &state.filter;
        let (came, went, new_only, bursts) = (dispatch.clone(), dispatch.clone(), dispatch.clone(), dispatch.clone());
        let spans: Vec<(String, View)> = [Span::Quarter, Span::Hour, Span::Connected]
            .into_iter()
            .map(|span| {
                let dispatch = dispatch.clone();
                (
                    span.name().to_string(),
                    RadioButton::new()
                        .mark(span)
                        .group_name("activity-span")
                        .is_checked(state.span == span)
                        .on_checked(move |_: bool| dispatch.emit(ShowSpan(span)))
                        .content(text(span_label(span, l10n))),
                )
            })
            .collect();
        let choices = StackPanel::new().children((
            shown(ActivityMark::Came, l10n.activity_came(), filter.came, move |on| came.emit(ShowCame(on))),
            shown(ActivityMark::Went, l10n.activity_went(), filter.went, move |on| went.emit(ShowWent(on))),
            shown(ActivityMark::NewOnly, l10n.activity_new_only(), filter.new_only, move |on| {
                new_only.emit(NewOnly(on))
            }),
            shown(ActivityMark::Bursts, l10n.activity_bursts(), filter.bursts, move |on| {
                bursts.emit(ShowBursts(on))
            }),
            separator(palette).margin(Thickness::xy(0.0, space::Control)),
            StackPanel::new().children((View::keyed_fragment(spans),)),
        ));
        Button::new()
            .mark(ActivityMark::Menu)
            .style(ButtonStyle::Subtle)
            .content(icon!(more_horizontal).size(size::Icon).build())
            .flyout_with(Flyout::rich(choices).placement(FlyoutPlacement::BottomEdgeAlignedRight))
    }

    fn header(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        Grid::new()
            .columns([GridLength::Star(1.0), GridLength::Auto])
            .children((
                Border::new().grid_column(0).content(page_title(l10n.activity_title())),
                Border::new()
                    .grid_column(1)
                    .vertical_alignment(VerticalAlignment::Center)
                    .content(Self::menu(state, dispatch, l10n, palette)),
            ))
    }

    fn range_bar(view: &ActivityView, state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let searched = dispatch.clone();
        let cleared = dispatch.clone();
        let where_: View = match view.scatter.area {
            Some(_) => StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((
                    caption(l10n.activity_paused(format::clock(view.from), format::clock(view.to)))
                        .vertical_alignment(VerticalAlignment::Center),
                    command_button(ActivityMark::ClearArea, l10n.activity_live(), None, true, move || {
                        cleared.emit(ClearArea)
                    }),
                )),
            None => caption(
                view.history_since
                    .map(|at| l10n.activity_history_since(format::clock(at)))
                    .unwrap_or_default(),
            )
            .foreground(palette.secondary_text)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        };
        Grid::new()
            .columns([GridLength::Star(1.0), GridLength::Auto])
            .margin(Thickness::xy(space::Cell, space::Control))
            .children((
                Border::new().grid_column(0).vertical_alignment(VerticalAlignment::Center).content(where_),
                TextBox::new()
                    .mark(ActivityMark::Search)
                    .grid_column(1)
                    .width(SearchBox::Width)
                    .text(state.filter.text.clone())
                    .placeholder_text(l10n.activity_search())
                    .on_text_changed(move |text: String| searched.emit(Search(text))),
            ))
    }

    fn chart(view: &ActivityView, state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let scatter = &view.scatter;
        let (picked, cleared, hovered) = (dispatch.clone(), dispatch.clone(), dispatch.clone());
        let plot = View::component::<ScatterPlot>(Plotted {
            scatter: scatter.clone(),
            card: state.hovered.as_ref().map(|row| (row.key(), card(row, l10n, palette))),
            palette,
            on_pick: Callback::new(move |area: Area| picked.emit(PickArea(area))),
            on_clear: Callback::new(move |()| cleared.emit(ClearArea)),
            on_hover: Callback::new(move |key| hovered.emit(Hover(key))),
        });
        Grid::new()
            .columns([GridLength::Auto, GridLength::Star(1.0)])
            .children((
                Border::new()
                    .grid_column(0)
                    .vertical_alignment(VerticalAlignment::Top)
                    .content(labels(scatter, |lived| lasted(l10n, lived), palette)),
                Border::new().grid_column(1).content(plot),
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
        let header = Self::header(state, dispatch, l10n, palette);
        let Load::Ready(view) = &state.view else {
            return page_frame(header, loading(), status_text(l10n.activity_loading(), palette), palette);
        };

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
                    earlier: view.earlier,
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
                    .content(Self::chart(view, state, dispatch, l10n, palette)),
                Border::new()
                    .grid_row(1)
                    .content(Self::range_bar(view, state, dispatch, l10n, palette)),
                Border::new().grid_row(2).content(list),
            ));

        let (from, to) = (format::clock(view.from), format::clock(view.to));
        let status = if view.lost > 0 {
            l10n.activity_range_lost(from, to, view.came as i64, view.went as i64, view.lost as i64)
        } else {
            l10n.activity_range(from, to, view.came as i64, view.went as i64)
        };
        page_frame(header, body, status_text(status, palette), palette)
    }
}
