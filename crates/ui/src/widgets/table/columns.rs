use std::collections::BTreeMap;
use std::rc::Rc;

use guinea::Mark;
use windows_reactor::{Color, GridLength, TextBlock, View};

pub(super) struct Space;

#[expect(non_upper_case_globals)]
impl Space {
    pub(super) const MinColumn: f64 = 24.0;
    pub(super) const CellInset: f64 = 12.0;
    pub(super) const HeaderInset: f64 = 8.0;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnWidths(BTreeMap<&'static str, f64>);

impl ColumnWidths {
    pub fn get(&self, id: &str) -> Option<f64> {
        self.0.get(id).copied()
    }

    pub fn apply(&mut self, drag: Resized) {
        self.0.insert(drag.column, drag.width);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resized {
    pub column: &'static str,
    pub width: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnOrder(Vec<&'static str>);

impl ColumnOrder {
    pub fn new(names: Vec<&'static str>) -> Self {
        Self(names)
    }

    pub fn names(&self) -> &[&'static str] {
        &self.0
    }

    pub fn apply(&mut self, moved: Reordered) {
        self.0 = moved.order;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reordered {
    pub order: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub row_height: f64,
    pub hovered: Color,
    pub selected: Color,
    pub inset: (f64, f64),
    pub radius: f64,
    pub separator: Color,
    pub rule: Color,
}

impl Default for Look {
    fn default() -> Self {
        let grey = |a| Color {
            a,
            r: 128,
            g: 128,
            b: 128,
        };

        Self {
            row_height: 32.0,
            hovered: grey(24),
            selected: grey(40),
            inset: (4.0, 1.0),
            radius: 4.0,
            separator: grey(48),
            rule: grey(48),
        }
    }
}

pub struct ColumnSpec<T, C> {
    pub id: C,
    pub header: Rc<dyn Fn() -> View>,
    pub initial_width: f64,
    pub min_width: f64,
    pub sortable: bool,
    pub flush: bool,
    pub fill: bool,
    pub cell: Rc<dyn Fn(&T) -> View>,
}

impl<T, C: Mark> ColumnSpec<T, C> {
    pub fn new(id: C, header: impl Into<String>, initial_width: f64, cell: impl Fn(&T) -> View + 'static) -> Self {
        let header = header.into();
        Self::new_with_header(id, move || TextBlock::new().text(header.clone()).into(), initial_width, cell)
    }

    pub fn new_with_header(
        id: C,
        header: impl Fn() -> View + 'static,
        initial_width: f64,
        cell: impl Fn(&T) -> View + 'static,
    ) -> Self {
        Self {
            id,
            header: Rc::new(header),
            initial_width,
            min_width: Space::MinColumn,
            sortable: false,
            flush: false,
            fill: false,
            cell: Rc::new(cell),
        }
    }

    pub fn min_width(mut self, min_width: f64) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn flush(mut self) -> Self {
        self.flush = true;
        self
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    pub(super) fn inset(&self) -> f64 {
        if self.flush { 0.0 } else { Space::CellInset }
    }
}

pub(super) fn width_of<T, C: Mark>(widths: &ColumnWidths, column: &ColumnSpec<T, C>) -> f64 {
    widths
        .get(column.id.name())
        .unwrap_or(column.initial_width)
        .max(column.min_width)
}

pub(super) struct Laid {
    pub(super) slots: Vec<usize>,
    pub(super) lengths: Vec<GridLength>,
    pub(super) least: f64,
}

impl Laid {
    pub(super) fn new<T, C: Mark>(columns: &[ColumnSpec<T, C>], placed: &[usize], widths: &ColumnWidths) -> Self {
        let lengths = placed
            .iter()
            .map(|&at| &columns[at])
            .map(|column| {
                if column.fill {
                    GridLength::Star(1.0)
                } else {
                    GridLength::Pixel(width_of(widths, column))
                }
            })
            .collect();
        let least = columns
            .iter()
            .map(|column| if column.fill { column.min_width } else { width_of(widths, column) })
            .sum();
        Self {
            slots: slots(placed),
            lengths,
            least,
        }
    }
}

pub(super) fn placed<T, C: Mark>(columns: &[ColumnSpec<T, C>], order: &ColumnOrder) -> Vec<usize> {
    let mut placed: Vec<usize> = (0..columns.len()).collect();
    if let Some((_, rest)) = placed.split_first_mut() {
        rest.sort_by_key(|&at| {
            let name = columns[at].id.name();
            order
                .names()
                .iter()
                .position(|named| *named == name)
                .unwrap_or(usize::MAX)
        });
    }
    placed
}

fn slots(placed: &[usize]) -> Vec<usize> {
    let mut slots = vec![0; placed.len()];
    for (slot, &at) in placed.iter().enumerate() {
        slots[at] = slot;
    }
    slots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum Col {
        Name,
        Size,
        Kind,
        Date,
    }

    impl Mark for Col {
        fn name(&self) -> &'static str {
            match self {
                Col::Name => "Name",
                Col::Size => "Size",
                Col::Kind => "Kind",
                Col::Date => "Date",
            }
        }
    }

    fn columns() -> Vec<ColumnSpec<(), Col>> {
        [Col::Name, Col::Size, Col::Kind, Col::Date]
            .into_iter()
            .map(|id| ColumnSpec::new(id, id.name(), 100.0, |_: &()| View::empty()))
            .collect()
    }

    #[test]
    fn with_no_order_the_columns_go_as_given() {
        assert_eq!(placed(&columns(), &ColumnOrder::default()), [0, 1, 2, 3]);
    }

    #[test]
    fn the_order_places_every_column_but_the_first() {
        let order = ColumnOrder::new(vec!["Date", "Name", "Kind", "Size"]);
        assert_eq!(placed(&columns(), &order), [0, 3, 2, 1]);
    }

    #[test]
    fn a_column_the_order_does_not_name_goes_after_the_named() {
        let order = ColumnOrder::new(vec!["Kind", "Gone"]);
        assert_eq!(placed(&columns(), &order), [0, 2, 1, 3]);
    }

    #[test]
    fn slots_are_where_each_column_was_placed() {
        assert_eq!(slots(&[0, 3, 1, 2]), [0, 2, 3, 1]);
    }
}
