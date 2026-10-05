use std::ops::Range;
use std::rc::Rc;

use guinea::Mark;
use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, CornerRadius, Grid, HorizontalAlignment,
    PointerEventInfo, Thickness, VerticalAlignment, View, ViewContext, VirtualSource, keyed,
};

use super::columns::{ColumnSpec, Laid, Look};

struct Rows;

#[expect(non_upper_case_globals)]
impl Rows {
    const Bucket: usize = 16;
}

pub(super) enum Pointer {
    Entered,
    Exited,
}

pub(super) fn rows_source<T: 'static, C: Mark>(
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T, C>>,
    laid: Laid,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    span: Option<Range<usize>>,
    look: Look,
) -> VirtualSource {
    let claimed = rows.len().next_multiple_of(Rows::Bucket);
    let rows = Rc::new(rows);
    let columns = Rc::new(columns);
    let laid = Rc::new(laid);
    let (selected, on_select) = match selection {
        Some((at, callback)) => (at, Some(callback)),
        None => (None, None),
    };
    let painted = span.or_else(|| selected.map(|at| at..at + 1));

    VirtualSource::new(
        claimed as u64,
        claimed,
        |index| index,
        move |index| match rows.get(index) {
            Some(row) => View::component::<Pointed>(Line {
                cells: row_view(row, &columns, &laid),
                index,
                selected: painted
                    .as_ref()
                    .filter(|run| run.contains(&index))
                    .map(|run| Run {
                        above: index > run.start,
                        below: index + 1 < run.end,
                    }),
                on_select: on_select.clone(),
                look,
            }),
            None => Border::new().into(),
        },
    )
}

#[derive(Clone, PartialEq)]
struct Line {
    cells: View,
    index: usize,
    selected: Option<Run>,
    on_select: Option<Callback<Option<usize>>>,
    look: Look,
}

#[derive(Clone, Copy, PartialEq)]
struct Run {
    above: bool,
    below: bool,
}

impl Run {
    const ALONE: Run = Run {
        above: false,
        below: false,
    };
}

struct Pointed(bool);

impl Component for Pointed {
    type Input = Line;
    type Message = Pointer;

    fn create(_input: &Line, _cx: &ComponentContext<Self>) -> Self {
        Self(false)
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        self.0 = matches!(message, Pointer::Entered);
    }

    fn input_changed(&mut self, _input: &Line, _cx: &ComponentContext<Self>) {}

    fn view(&self, line: &Line, cx: &mut ViewContext<Self>) -> View {
        let look = line.look;

        let (plate, run) = match line.selected {
            Some(run) => (look.selected, run),
            None if self.0 => (look.hovered, Run::ALONE),
            None => (Color::transparent(), Run::ALONE),
        };

        let (across, down) = look.inset;
        let top = if run.above { 0.0 } else { down };
        let bottom = if run.below { 0.0 } else { down };
        let upper = if run.above { 0.0 } else { look.radius };
        let lower = if run.below { 0.0 } else { look.radius };

        let layered = Grid::new().children((
            Border::new()
                .margin(Thickness::new(across, top, across, bottom))
                .corner_radius(CornerRadius::new(upper, upper, lower, lower))
                .background(plate),
            line.cells.clone(),
        ));

        let row = Border::new()
            .min_height(look.row_height)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .background(Color::transparent())
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Pointer::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Exited));

        match &line.on_select {
            Some(on_select) => {
                let on_select = on_select.clone();
                let index = line.index;
                row.on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                    on_select.call(Some(index));
                }))
                .content(layered)
                .into()
            }
            None => row.content(layered).into(),
        }
    }
}

fn row_view<T, C: Mark>(row: &T, columns: &[ColumnSpec<T, C>], laid: &Laid) -> View {
    let cells = columns.iter().enumerate().map(|(at, column)| {
        let cell = Border::new()
            .automation_id(column.id.name())
            .padding(Thickness::xy(column.inset(), 0.0))
            .grid_column(laid.slots[at] as i32)
            .vertical_alignment(VerticalAlignment::Center)
            .content((column.cell)(row));
        keyed(column.id.name(), cell)
    });

    Grid::new()
        .columns(laid.lengths.clone())
        .min_width(laid.least)
        .keyed_children(cells)
        .into()
}
