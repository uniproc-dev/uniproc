use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityState, ActivityView, ClearArea, Hide, Hover, NewOnly, Only, Pick, PickArea, Search, ShowCame, ShowSeries,
    ShowSpan, ShowWent, Span, Unhide,
};
use app_contracts::features::agents::ProcessInstance;
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use guinea::Mark;
use guinea_widgets::chart::scatter::{Scatter, ScatterEvent};
use windows_reactor::{
    keyed, Border, Button, ButtonStyle, Callback, CheckBox, Flyout, FlyoutExt, FlyoutPlacement, Grid, GridLength,
    HorizontalAlignment, KeyedView, Orientation, RadioButton, ScrollBarVisibility, ScrollViewer, StackPanel, TextBox,
    Thickness, VerticalAlignment, View,
};

use super::components::card::card;
use super::components::lifetimes::{acts, options, series, Act};
use super::components::picks::picked;
use super::components::rows::{rows, RowActs, Rows};
use super::marks::ActivityMark;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::button::command_button;
use crate::widgets::nothing::nothing;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::separator;
use crate::widgets::text::{caption, text};

struct SearchBox;

#[expect(non_upper_case_globals)]
impl SearchBox {
    const Width: f64 = 260.0;
}

struct Plot;

#[expect(non_upper_case_globals)]
impl Plot {
    const Height: f64 = 220.0;
    const CardGap: f64 = 12.0;
}

pub enum ActivityPageMsg {
    Toggle(ProcessInstance),
    Only(Pick),
    Hide(Pick),
}

#[derive(Default)]
pub struct ActivityPage {
    expanded: Rc<HashSet<ProcessInstance>>,
    chart: Scatter<ProcessInstance>,
}

fn shown(mark: impl Mark, label: String, checked: bool, on_change: impl Fn(bool) + 'static) -> View {
    CheckBox::new()
        .mark(mark)
        .is_checked(checked)
        .on_is_checked_changed(move |checked: Option<bool>| on_change(checked == Some(true)))
        .content(text(label))
        .into()
}

fn span_label(span: Span, l10n: &L10n) -> String {
    match span {
        Span::Quarter => l10n.activity_span_quarter(),
        Span::Hour => l10n.activity_span_hour(),
        Span::Connected => l10n.activity_span_connected(),
    }
}

impl ActivityPage {
    pub fn update(&mut self, message: ActivityPageMsg, dispatch: &Dispatch) {
        match message {
            ActivityPageMsg::Toggle(key) => {
                let expanded = Rc::make_mut(&mut self.expanded);
                if !expanded.remove(&key) {
                    expanded.insert(key);
                }
            }
            ActivityPageMsg::Only(pick) => dispatch.emit(Only(Some(pick))),
            ActivityPageMsg::Hide(pick) => dispatch.emit(Hide(pick)),
        }
    }

    fn menu(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let filter = &state.filter;
        let (came, went, new_only, series) = (dispatch.clone(), dispatch.clone(), dispatch.clone(), dispatch.clone());
        let spans: Vec<KeyedView> = [Span::Quarter, Span::Hour, Span::Connected]
            .into_iter()
            .map(|span| {
                let dispatch = dispatch.clone();
                keyed(
                    span.name(),
                    RadioButton::new()
                        .mark(span)
                        .group_name("activity-span")
                        .is_checked(state.span == span)
                        .on_checked(move |_: Option<bool>| dispatch.emit(ShowSpan(span)))
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
            shown(ActivityMark::Series, l10n.activity_series(), filter.series, move |on| {
                series.emit(ShowSeries(on))
            }),
            separator(palette).margin(Thickness::xy(0.0, space::Control)),
            StackPanel::new().keyed_children(spans),
        ));
        Button::new()
            .mark(ActivityMark::Menu)
            .style(ButtonStyle::Subtle)
            .content(icon!(more_horizontal).size(size::Icon).build_element())
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
            .into()
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
                ))
                .into(),
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
                TextBox::new(&state.filter.text)
                    .mark(ActivityMark::Search)
                    .grid_column(1)
                    .width(SearchBox::Width)
                    .placeholder_text(l10n.activity_search())
                    .on_text_changed(move |text: Rc<str>| searched.emit(Search(text.to_string()))),
            ))
            .into()
    }

    fn chart(&self, view: &ActivityView, state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let scatter = view.scatter.clone();
        self.chart.publish(series(&scatter, palette), options(&scatter, l10n, palette));
        let dispatch = dispatch.clone();
        let plot = self.chart.view(move |event: ScatterEvent<ProcessInstance>| {
            for act in acts(&event, &scatter) {
                match act {
                    Act::Hover(key) => dispatch.emit(Hover(key)),
                    Act::Pick(area) => dispatch.emit(PickArea(area)),
                    Act::Clear => dispatch.emit(ClearArea),
                }
            }
        });
        let mut layers: Vec<View> = vec![Border::new().mark(ActivityMark::Scatter).height(Plot::Height).content(plot).into()];
        if let (Some(row), Some(hit)) = (&state.hovered, self.chart.hovered()) {
            layers.push(
                Border::new()
                    .horizontal_alignment(HorizontalAlignment::Left)
                    .vertical_alignment(VerticalAlignment::Top)
                    .margin(Thickness::new(
                        f64::from(hit.x) + Plot::CardGap,
                        f64::from(hit.y) + Plot::CardGap,
                        0.0,
                        0.0,
                    ))
                    .content(card(row, l10n, palette))
                    .into(),
            );
        }
        Grid::new().children(layers).into()
    }

    pub fn view(
        &self,
        state: &ActivityState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        acts: RowActs,
    ) -> View {
        let header = Self::header(state, dispatch, l10n, palette);
        let Load::Ready(view) = &state.view else {
            return page_frame(header, loading(), status_text(l10n.activity_loading(), palette), palette);
        };

        let (all, unhide) = (dispatch.clone(), dispatch.clone());
        let chips = picked(
            &state.filter,
            l10n,
            Callback::new(move |()| all.emit(Only(None))),
            Callback::new(move |pick: Pick| unhide.emit(Unhide(pick))),
        );
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
                    acts,
                }))
                .into()
        };

        let body = Grid::new()
            .rows([GridLength::Auto, GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
            .children((
                Border::new()
                    .grid_row(0)
                    .margin(Thickness::new(space::Cell, space::Card, space::Cell, 0.0))
                    .content(self.chart(view, state, dispatch, l10n, palette)),
                Border::new()
                    .grid_row(1)
                    .content(Self::range_bar(view, state, dispatch, l10n, palette)),
                Border::new().grid_row(2).content(chips.unwrap_or_else(nothing)),
                Border::new().grid_row(3).content(list),
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
