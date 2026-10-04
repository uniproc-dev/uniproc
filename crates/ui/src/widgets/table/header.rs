use std::cell::Cell;
use std::rc::Rc;

use guinea::Mark;
use guinea_widgets::resize::resize_handle;
use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, CornerRadius, Grid, GridLength, PointerEventInfo,
    TextBlock, Thickness, View, ViewContext,
};

use super::columns::{ColumnSpec, Reordered, Resized, Space};
use super::rows::Pointer;
use super::sort::SortState;

pub(super) struct HeaderCell<'a, T, C> {
    pub(super) column: &'a ColumnSpec<T, C>,
    pub(super) sort_state: Option<&'a SortState<C>>,
    pub(super) on_sort: Option<&'a Callback<C>>,
    pub(super) sort_indicator: Option<&'a Rc<dyn Fn(bool) -> Option<View>>>,
    pub(super) moving: Option<Moving>,
    pub(super) hovered: Color,
    pub(super) rounded: (f64, f64),
    pub(super) railed: bool,
}

pub(super) fn header_cell<T, C: Mark + Clone + PartialEq + 'static>(cell: HeaderCell<'_, T, C>) -> View {
    let HeaderCell {
        column,
        sort_state,
        on_sort,
        sort_indicator,
        moving,
        hovered,
        rounded,
        railed,
    } = cell;
    let active = sort_state.filter(|s| column.sortable && s.field_id.as_ref() == Some(&column.id));

    let base = (column.header)();
    let indicator = active.and_then(|state| match sort_indicator {
        Some(render) => render(state.descending),
        None => Some(
            TextBlock::new()
                .text(if state.descending { "▼" } else { "▲" })
                .into(),
        ),
    });
    let content = match indicator {
        Some(indicator) => {
            Grid::new()
                .columns([GridLength::Star(1.0), GridLength::Auto])
                .children((
                    Border::new().grid_column(0).content(base),
                    Border::new().grid_column(1).content(indicator),
                ))
                .into()
        }
        None => base,
    };

    let padding = Thickness::xy(column.inset(), Space::HeaderInset);

    let sorts = on_sort.filter(|_| column.sortable).cloned();
    if sorts.is_none() && moving.is_none() {
        return Border::new()
            .automation_id(column.id.name())
            .padding(padding)
            .content(content)
            .into();
    }

    View::component::<PointedHeading<C>>(Heading {
        content: Border::new().padding(padding).content(content).into(),
        column: column.id.clone(),
        on_sort: sorts,
        moving,
        hovered,
        rounded,
        railed,
    })
}

#[derive(Clone, PartialEq)]
struct Heading<C> {
    content: View,
    column: C,
    on_sort: Option<Callback<C>>,
    moving: Option<Moving>,
    hovered: Color,
    rounded: (f64, f64),
    railed: bool,
}

#[derive(Clone, PartialEq)]
pub(super) struct Moving {
    pub(super) order: Vec<&'static str>,
    pub(super) at: usize,
    pub(super) left: Option<f64>,
    pub(super) right: Option<f64>,
    pub(super) on_reorder: Callback<Reordered>,
}

impl Moving {
    fn step(&self, delta: f64) -> Option<(Vec<&'static str>, f64)> {
        let (to, shift) = match (self.left, self.right) {
            (_, Some(right)) if delta > right / 2.0 => (self.at + 1, right),
            (Some(left), _) if delta < -left / 2.0 => (self.at - 1, -left),
            _ => return None,
        };

        let mut order = self.order.clone();
        order.swap(self.at, to);
        Some((order, shift))
    }
}

#[derive(Default)]
struct Drag {
    anchor: Cell<Option<f64>>,
    moved: Cell<bool>,
    sent_from: Cell<Option<usize>>,
}

#[expect(non_upper_case_globals)]
impl Drag {
    const Threshold: f64 = 4.0;
}

struct PointedHeading<C> {
    hovered: bool,
    drag: Rc<Drag>,
    _column: std::marker::PhantomData<fn() -> C>,
}

impl<C: Mark + Clone + PartialEq + 'static> Component for PointedHeading<C> {
    type Input = Heading<C>;
    type Message = Pointer;

    fn create(_input: &Heading<C>, _cx: &ComponentContext<Self>) -> Self {
        Self {
            hovered: false,
            drag: Rc::default(),
            _column: std::marker::PhantomData,
        }
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        self.hovered = matches!(message, Pointer::Entered);
    }

    fn view(&self, heading: &Heading<C>, cx: &mut ViewContext<Self>) -> View {
        let plate = if self.hovered { heading.hovered } else { Color::transparent() };
        let (left, right) = heading.rounded;
        let rail = if heading.railed { 1.0 } else { 0.0 };

        let layered = Grid::new().children((
            Border::new()
                .margin(Thickness::new(0.0, 0.0, rail, 0.0))
                .corner_radius(CornerRadius::new(left, right, 0.0, 0.0))
                .background(plate),
            heading.content.clone(),
        ));

        let column = heading.column.clone();
        let on_sort = heading.on_sort.clone();
        let released = self.drag.clone();

        let cell = Border::new()
            .automation_id(heading.column.name())
            .background(Color::transparent())
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Pointer::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Exited))
            .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                released.anchor.set(None);
                if released.moved.replace(false) {
                    return;
                }
                if let Some(on_sort) = &on_sort {
                    on_sort.call(column.clone());
                }
            }));

        let Some(moving) = heading.moving.clone() else {
            return cell.content(layered).into();
        };

        let pressed = self.drag.clone();
        let dragged = self.drag.clone();
        let lost = self.drag.clone();
        cell.capture_pointer_on_press(true)
            .on_pointer_capture_lost(move || lost.anchor.set(None))
            .on_pointer_pressed(Callback::new(move |info: PointerEventInfo| {
                pressed.anchor.set(Some(info.window_x));
                pressed.moved.set(false);
                pressed.sent_from.set(None);
            }))
            .on_pointer_moved(Callback::new(move |info: PointerEventInfo| {
                let Some(anchor) = dragged.anchor.get().filter(|_| info.is_left_button_pressed) else {
                    return;
                };
                let delta = info.window_x - anchor;
                if delta.abs() > Drag::Threshold {
                    dragged.moved.set(true);
                }
                if !dragged.moved.get() || dragged.sent_from.get() == Some(moving.at) {
                    return;
                }
                if let Some((order, shift)) = moving.step(delta) {
                    moving.on_reorder.call(Reordered { order });
                    dragged.anchor.set(Some(anchor + shift));
                    dragged.sent_from.set(Some(moving.at));
                }
            }))
            .content(layered)
            .into()
    }
}

pub(super) fn handle<T, C: Mark>(column: &ColumnSpec<T, C>, width: f64, rail: Color, on_resize: Callback<Resized>) -> View {
    let id = column.id.name();
    resize_handle(width, move |width| {
        on_resize.call(Resized { column: id, width });
    })
    .min(column.min_width)
    .rail(rail)
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving(at: usize, left: Option<f64>, right: Option<f64>) -> Moving {
        Moving {
            order: vec!["Name", "Size", "Kind", "Date"],
            at,
            left,
            right,
            on_reorder: Callback::new(|_: Reordered| {}),
        }
    }

    #[test]
    fn a_header_trades_places_past_half_its_neighbour() {
        let middle = moving(2, Some(80.0), Some(60.0));
        assert_eq!(middle.step(29.0), None);
        assert_eq!(middle.step(31.0), Some((vec!["Name", "Size", "Date", "Kind"], 60.0)));
        assert_eq!(middle.step(-39.0), None);
        assert_eq!(middle.step(-41.0), Some((vec!["Name", "Kind", "Size", "Date"], -80.0)));
    }

    #[test]
    fn a_header_does_not_pass_a_neighbour_it_may_not() {
        let second = moving(1, None, Some(60.0));
        assert_eq!(second.step(-500.0), None);
        let last = moving(3, Some(60.0), None);
        assert_eq!(last.step(500.0), None);
    }
}
