use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityRow, ActivityState, ActivityView, ClearArea, Filter, Group, Hide, Hover, NewGroup, NewOnly, Only,
    PickArea, PutInGroup, Search, ShowCame, ShowSeries, ShowSpan, ShowWent, Span,
};
use app_contracts::features::agents::ProcessInstance;
use guicons::icon;
use guinea::prelude::{Dispatch, Load};
use guinea::winui::MarkExt;
use guinea::Mark;
use guinea_widgets::chart::scatter::{Scatter, ScatterEvent};
use windows_reactor::{
    keyed, Border, Button, ButtonStyle, Callback, Canvas, CheckBox, Flyout, FlyoutExt, FlyoutPlacement, Grid, GridLength,
    HorizontalAlignment, KeyedView, Orientation, PointerEventInfo, RadioButton, ScrollBarVisibility, ScrollViewer,
    StackPanel, TextBox, Thickness, VerticalAlignment, View,
};

use super::components::card::card;
use super::components::legend::legend;
use super::components::lifetimes::{acts, options, Act, Plotted};
use super::components::picks::{pick_lines, picked, PickCommand};
use super::components::rows::{rows, Rows};
use super::marks::ActivityMark;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::button::command_button;
use crate::widgets::nothing::nothing;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::popup_menu::{popup_menu, PopupMenu};
use crate::widgets::selection::{Pinned, SelectionMark};
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

#[derive(Clone)]
pub enum ActivityPageMsg {
    Press(ProcessInstance),
    Pick(PickCommand),
    MenuAnchor { x: f64, y: f64 },
    MenuDismiss,
}

struct RowMenu {
    x: f64,
    y: f64,
    row: ProcessInstance,
}

#[derive(Default)]
pub struct ActivityPage {
    anchor: Option<(f64, f64)>,
    menu: Option<RowMenu>,
    expanded: Rc<HashSet<ProcessInstance>>,
    selected: Cell<Option<ProcessInstance>>,
    pinned: RefCell<Pinned<ActivityRow, ProcessInstance>>,
    asked: RefCell<Option<(Span, Filter)>>,
    chart: Scatter<ProcessInstance>,
    plotted: RefCell<Option<Rc<Plotted>>>,
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
        Span::HalfMinute => l10n.activity_span_half_minute(),
        Span::FiveMinutes => l10n.activity_span_five_minutes(),
        Span::Quarter => l10n.activity_span_quarter(),
        Span::HalfHour => l10n.activity_span_half_hour(),
        Span::Hour => l10n.activity_span_hour(),
        Span::Day => l10n.activity_span_day(),
        Span::Connected => l10n.activity_span_connected(),
    }
}

impl ActivityPage {
    pub fn update(&mut self, message: ActivityPageMsg, dispatch: &Dispatch) {
        match message {
            ActivityPageMsg::Press(row) => {
                self.selected.set(Some(row));
                if let Some((x, y)) = self.anchor.take() {
                    self.menu = Some(RowMenu { x, y, row });
                    return;
                }
                let expanded = Rc::make_mut(&mut self.expanded);
                if !expanded.remove(&row) {
                    expanded.insert(row);
                }
            }
            ActivityPageMsg::Pick(command) => {
                self.menu = None;
                match command {
                    PickCommand::Only(pick) => dispatch.emit(Only(Some(pick))),
                    PickCommand::Hide(pick) => dispatch.emit(Hide(pick)),
                    PickCommand::Put { group, rule } => dispatch.emit(PutInGroup { group, rule }),
                    PickCommand::New(rule) => dispatch.emit(NewGroup(rule)),
                }
            }
            ActivityPageMsg::MenuAnchor { x, y } => self.anchor = Some((x, y)),
            ActivityPageMsg::MenuDismiss => {
                self.anchor = None;
                self.menu = None;
            }
        }
    }

    fn row_menu(
        &self,
        rows: &[ActivityRow],
        groups: &[Group],
        l10n: &L10n,
        palette: Palette,
        forward: &Callback<ActivityPageMsg>,
    ) -> Option<View> {
        let menu = self.menu.as_ref()?;
        let row = rows.iter().find(|row| row.key() == menu.row)?;
        let lines = pick_lines(row.picks(), groups, l10n, palette);
        if lines.is_empty() {
            return None;
        }
        let (picked, dismissed) = (forward.clone(), forward.clone());
        Some(popup_menu(PopupMenu {
            x: menu.x,
            y: menu.y,
            lines,
            card: ActivityMark::RowMenu,
            backdrop: ActivityMark::RowMenuBackdrop,
            palette,
            on_command: Callback::new(move |command: PickCommand| picked.call(ActivityPageMsg::Pick(command))),
            on_dismiss: Callback::new(move |()| dismissed.call(ActivityPageMsg::MenuDismiss)),
        }))
    }

    fn menu(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette, manage: Callback<()>) -> View {
        let filter = &state.filter;
        let (came, went, new_only, series) = (dispatch.clone(), dispatch.clone(), dispatch.clone(), dispatch.clone());
        let spans: Vec<KeyedView> = Span::ALL
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
            separator(palette).margin(Thickness::xy(0.0, space::Control)),
            Button::new()
                .mark(ActivityMark::ManageGroups)
                .style(ButtonStyle::Subtle)
                .on_click(move || manage.call(()))
                .content(text(l10n.activity_menu_manage_groups())),
        ));
        Button::new()
            .mark(ActivityMark::Menu)
            .style(ButtonStyle::Subtle)
            .content(icon!(more_horizontal).size(size::Icon).build_element())
            .flyout_with(Flyout::rich(choices).placement(FlyoutPlacement::BottomEdgeAlignedRight))
    }

    fn header(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette, manage: Callback<()>) -> View {
        Grid::new()
            .columns([GridLength::Star(1.0), GridLength::Auto])
            .children((
                Border::new().grid_column(0).content(page_title(l10n.activity_title())),
                Border::new()
                    .grid_column(1)
                    .vertical_alignment(VerticalAlignment::Center)
                    .content(Self::menu(state, dispatch, l10n, palette, manage)),
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

    fn chart(&self, view: &ActivityView, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let scatter = view.scatter.clone();
        let plotted = {
            let mut kept = self.plotted.borrow_mut();
            match &*kept {
                Some(plotted) if plotted.holds(&scatter, palette) => plotted.clone(),
                _ => {
                    let plotted = Rc::new(Plotted::new(scatter.clone(), palette));
                    *kept = Some(plotted.clone());
                    plotted
                }
            }
        };
        self.chart.publish(plotted, options(&scatter, l10n, palette));
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
        Border::new().mark(ActivityMark::Scatter).height(Plot::Height).content(plot).into()
    }

    fn hover_card(&self, state: &ActivityState, l10n: &L10n, palette: Palette) -> Option<View> {
        let (row, hit) = (state.hovered.as_ref()?, self.chart.hovered()?);
        let placed = Border::new()
            .canvas_left(f64::from(hit.x) + Plot::CardGap)
            .canvas_top(f64::from(hit.y) + Plot::CardGap)
            .content(card(row, l10n, palette));
        Some(Canvas::new().children((placed,)).into())
    }

    pub fn view(
        &self,
        state: &ActivityState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ActivityPageMsg>,
        manage_groups: Callback<()>,
    ) -> View {
        let header = Self::header(state, dispatch, l10n, palette, manage_groups);
        let Load::Ready(view) = &state.view else {
            return page_frame(header, loading(), status_text(l10n.activity_loading(), palette), palette);
        };

        let all = dispatch.clone();
        let chips = picked(&state.filter, l10n, Callback::new(move |()| all.emit(Only(None))));
        let asked = Some((state.span, state.filter.clone()));
        if *self.asked.borrow() != asked {
            self.selected.set(None);
            self.asked.replace(asked);
        }
        let placed = self.pinned.borrow_mut().place(&view.rows, self.selected.get(), ActivityRow::key);
        let list: View = if placed.is_empty() {
            text(l10n.activity_empty())
                .mark(ActivityMark::Empty)
                .horizontal_alignment(HorizontalAlignment::Center)
                .margin(Thickness::uniform(space::Section))
                .into()
        } else {
            let menu = self.row_menu(&placed, &state.groups, l10n, palette, &forward);
            let (pressed, anchored) = (forward.clone(), forward);
            let scroller = ScrollViewer::new()
                .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                .content(rows(Rows {
                    rows: Rc::from(placed),
                    earlier: view.earlier,
                    expanded: self.expanded.clone(),
                    selected: self.selected.get(),
                    l10n: l10n.clone(),
                    palette,
                    on_press: Callback::new(move |row: ProcessInstance| pressed.call(ActivityPageMsg::Press(row))),
                }));
            let mut layers: Vec<View> = vec![Border::new()
                .on_pointer_pressed(move |pointer: PointerEventInfo| {
                    anchored.call(if pointer.is_right_button_pressed {
                        ActivityPageMsg::MenuAnchor {
                            x: pointer.x,
                            y: pointer.y,
                        }
                    } else {
                        ActivityPageMsg::MenuDismiss
                    })
                })
                .content(scroller)
                .into()];
            layers.extend(menu);
            Grid::new().mark(SelectionMark::Keeper).children(layers).into()
        };

        let plot_margin = Thickness::new(space::Cell, space::Compact, space::Cell, 0.0);
        let mut layers: Vec<View> = vec![
            Border::new()
                .grid_row(0)
                .content(legend(&state.groups, &state.filter, &view.legend, dispatch, l10n, palette))
                .into(),
            Border::new()
                .grid_row(1)
                .margin(plot_margin)
                .content(self.chart(view, dispatch, l10n, palette))
                .into(),
            Border::new()
                .grid_row(2)
                .content(Self::range_bar(view, state, dispatch, l10n, palette))
                .into(),
            Border::new().grid_row(3).content(chips.unwrap_or_else(nothing)).into(),
            Border::new().grid_row(4).content(list).into(),
        ];
        if let Some(card) = self.hover_card(state, l10n, palette) {
            layers.push(Border::new().grid_row(1).margin(plot_margin).content(card).into());
        }
        let body = Grid::new()
            .rows([
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Star(1.0),
            ])
            .children(layers);

        let (from, to) = (format::clock(view.from), format::clock(view.to));
        let status = if view.lost > 0 {
            l10n.activity_range_lost(from, to, view.came as i64, view.went as i64, view.lost as i64)
        } else {
            l10n.activity_range(from, to, view.came as i64, view.went as i64)
        };
        page_frame(header, body, status_text(status, palette), palette)
    }
}
