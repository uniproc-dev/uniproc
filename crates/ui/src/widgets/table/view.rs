use std::ops::Range;
use std::rc::Rc;

use guinea::winui::MarkExt;
use guinea::Mark;
use guinea_widgets::resize::RESIZE_HANDLE_WIDTH;
use windows_reactor::{
    keyed, Border, Callback, Color, Grid, GridLength, HorizontalAlignment, IntoPayloadCallback, ItemsRepeater,
    PointerEventInfo, Rectangle, ScrollBarVisibility, ScrollViewer, Thickness, VerticalAlignment, View,
};

use super::columns::{placed, width_of, ColumnOrder, ColumnSpec, ColumnWidths, Laid, Look, Reordered, Resized};
use super::header::{handle, header_cell, HeaderCell, Moving};
use super::rows::rows_source;
use super::sort::SortState;

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TableMark {
    EmptyArea,
}

pub struct Table<T, C> {
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T, C>>,
    widths: ColumnWidths,
    on_resize: Option<Callback<Resized>>,
    order: ColumnOrder,
    on_reorder: Option<Callback<Reordered>>,
    sort: Option<(SortState<C>, Callback<C>)>,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    span: Option<Range<usize>>,
    sort_indicator: Option<Rc<dyn Fn(bool) -> Option<View>>>,
    look: Look,
    corner_radius: f64,
}

pub fn table<T: 'static, C: Mark + Clone + PartialEq>(rows: Vec<T>, columns: Vec<ColumnSpec<T, C>>) -> Table<T, C> {
    Table {
        widths: ColumnWidths::default(),
        rows,
        columns,
        on_resize: None,
        order: ColumnOrder::default(),
        on_reorder: None,
        sort: None,
        selection: None,
        span: None,
        sort_indicator: None,
        look: Look::default(),
        corner_radius: 0.0,
    }
}

impl<T: 'static, C: Mark + Clone + PartialEq + 'static> Table<T, C> {
    pub fn widths(mut self, widths: &ColumnWidths) -> Self {
        self.widths = widths.clone();
        self
    }

    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn on_resize(mut self, on_resize: impl IntoPayloadCallback<Resized>) -> Self {
        self.on_resize = Some(on_resize.into_payload_callback());
        self
    }

    pub fn order(mut self, order: &ColumnOrder) -> Self {
        self.order = order.clone();
        self
    }

    pub fn on_reorder(mut self, on_reorder: impl IntoPayloadCallback<Reordered>) -> Self {
        self.on_reorder = Some(on_reorder.into_payload_callback());
        self
    }

    pub fn sort(mut self, state: SortState<C>, on_sort: impl IntoPayloadCallback<C>) -> Self {
        self.sort = Some((state, on_sort.into_payload_callback()));
        self
    }

    pub fn selection(mut self, at: Option<usize>, on_select: impl IntoPayloadCallback<Option<usize>>) -> Self {
        self.selection = Some((at, on_select.into_payload_callback()));
        self
    }

    pub fn selected_span(mut self, rows: Range<usize>) -> Self {
        self.span = Some(rows);
        self
    }

    pub fn sort_indicator(mut self, render: impl Fn(bool) -> Option<View> + 'static) -> Self {
        self.sort_indicator = Some(Rc::new(render));
        self
    }

    pub fn build(self) -> View {
        let Self {
            rows,
            columns,
            widths,
            on_resize,
            order,
            on_reorder,
            sort,
            selection,
            span,
            sort_indicator,
            look,
            corner_radius,
        } = self;

        let (sort_state, on_sort) = match sort {
            Some((state, callback)) => (Some(state), Some(callback)),
            None => (None, None),
        };

        let placed = placed(&columns, &order);
        let laid = Laid::new(&columns, &placed, &widths);

        let last = columns.len().saturating_sub(1);
        let reaches_right = columns.iter().any(|column| column.fill);
        let mut header_cells: Vec<(String, View)> = Vec::with_capacity(columns.len() * 2);
        let mut handles: Vec<(String, View)> = Vec::with_capacity(columns.len());
        for (at, column) in columns.iter().enumerate() {
            let slot = laid.slots[at];
            let rounded = (
                if slot == 0 { corner_radius } else { 0.0 },
                if slot == last && reaches_right { corner_radius } else { 0.0 },
            );
            let railed = on_resize.is_some() && !column.fill;

            let moving = on_reorder
                .as_ref()
                .filter(|_| slot != 0 && !column.fill)
                .map(|on_reorder| {
                    let beside = |slot: usize| {
                        let neighbour = &columns[placed[slot]];
                        (slot != 0 && !neighbour.fill).then(|| width_of(&widths, neighbour))
                    };
                    Moving {
                        order: placed.iter().map(|&at| columns[at].id.name()).collect(),
                        at: slot,
                        left: beside(slot - 1),
                        right: (slot < last).then(|| beside(slot + 1)).flatten(),
                        on_reorder: on_reorder.clone(),
                    }
                });

            let cell = header_cell(HeaderCell {
                column,
                sort_state: sort_state.as_ref(),
                on_sort: on_sort.as_ref(),
                sort_indicator: sort_indicator.as_ref(),
                moving,
                hovered: look.hovered,
                rounded,
                railed,
            });

            header_cells.push((
                column.id.name().to_string(),
                Border::new().grid_column(slot as i32).content(cell).into(),
            ));

            if let Some(on_resize) = &on_resize
                && railed
            {
                let key = format!("{}/resize", column.id.name());
                handles.push((
                    key.clone(),
                    Border::new()
                        .automation_id(key)
                        .grid_column(slot as i32)
                        .width(RESIZE_HANDLE_WIDTH)
                        .horizontal_alignment(HorizontalAlignment::Right)
                        .margin(Thickness::new(0.0, 0.0, 0.5 - RESIZE_HANDLE_WIDTH / 2.0, 0.0))
                        .content(handle(column, width_of(&widths, column), look.separator, on_resize.clone()))
                        .into(),
                ));
            }
        }
        header_cells.extend(handles);

        let header = Grid::new()
            .columns(laid.lengths.clone())
            .min_width(laid.least)
            .grid_row(0)
            .keyed_children(header_cells.into_iter().map(|(key, cell)| keyed(key, cell)));

        let separator = Rectangle::new().fill(look.rule).height(1.0).grid_row(1);

        let on_deselect = selection.as_ref().map(|(_, on_select)| on_select.clone());

        let lines = ItemsRepeater::new()
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .virtual_source(rows_source(rows, columns, laid, selection, span, look));

        let content: View = match on_deselect {
            Some(on_deselect) => Grid::new()
                .children((
                    Border::new()
                        .mark(TableMark::EmptyArea)
                        .background(Color::transparent())
                        .horizontal_alignment(HorizontalAlignment::Stretch)
                        .vertical_alignment(VerticalAlignment::Stretch)
                        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                            on_deselect.call(None);
                        })),
                    lines,
                ))
                .into(),
            None => lines.into(),
        };

        let body = ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .grid_row(2)
            .content(content);

        Grid::new()
            .rows([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
            .children((header, separator, body))
            .into()
    }
}
