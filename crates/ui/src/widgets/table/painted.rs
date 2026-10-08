use std::rc::Rc;

use guinea::Mark;
use table::layout::Width;
use table::model::{Cell, Rgba, RowKey, Source};
use windows_reactor::{Callback, Color, View};

use super::columns::{width_of, CellFn, ColumnSpec, ColumnWidths, Look, PaintFn};
use crate::theme::Palette;

pub(super) type KeyFn<T> = Rc<dyn Fn(&T) -> RowKey>;

struct Rows<T> {
    rows: Vec<T>,
    paints: Vec<PaintFn<T>>,
    key: Option<KeyFn<T>>,
    height: f32,
}

impl<T> Rows<T> {
    fn at(&self, key: RowKey) -> Option<usize> {
        (0..self.rows.len()).find(|&at| self.key(at) == key)
    }
}

impl<T> Source for Rows<T> {
    fn len(&self) -> usize {
        self.rows.len()
    }

    fn key(&self, at: usize) -> RowKey {
        match &self.key {
            Some(key) => key(&self.rows[at]),
            None => at as RowKey,
        }
    }

    fn height(&self, _at: usize) -> f32 {
        self.height
    }

    fn columns(&self) -> usize {
        self.paints.len()
    }

    fn cell(&self, at: usize, column: usize, out: &mut Cell) {
        out.clear();
        (self.paints[column])(&self.rows[at], out);
    }
}

fn rgba(color: Color) -> Rgba {
    Rgba { r: color.r, g: color.g, b: color.b, a: color.a }
}

fn painted_look(look: Look, palette: Palette) -> table::Look {
    table::Look {
        tones: [
            rgba(palette.primary_text),
            rgba(palette.secondary_text),
            rgba(palette.tertiary_text),
            rgba(palette.disabled_text),
            rgba(palette.success),
        ],
        hovered: rgba(look.hovered),
        selected: rgba(look.selected),
        plate_inset: (look.inset.0 as f32, look.inset.1 as f32),
        plate_radius: look.radius as f32,
        ..table::Look::default()
    }
}

pub(super) struct Painted<T, C> {
    pub rows: Vec<T>,
    pub columns: Vec<ColumnSpec<T, C>>,
    pub placed: Vec<usize>,
    pub widths: ColumnWidths,
    pub key: Option<KeyFn<T>>,
    pub selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    pub look: Look,
    pub palette: Palette,
}

pub(super) fn painted_body<T: 'static, C: Mark>(painted: Painted<T, C>) -> View {
    let Painted { rows, columns, placed, widths, key, selection, look, palette } = painted;
    let paints = placed
        .iter()
        .filter_map(|&at| match &columns[at].cell {
            CellFn::Paint(paint) => Some(paint.clone()),
            CellFn::View(_) => None,
        })
        .collect();
    let laid: Rc<[Width]> = placed
        .iter()
        .map(|&at| &columns[at])
        .map(|column| {
            if column.fill {
                Width::Fill { min: column.min_width as f32 }
            } else {
                Width::Fixed(width_of(&widths, column) as f32)
            }
        })
        .collect();
    let rows = Rc::new(Rows { rows, paints, key, height: look.row_height as f32 });
    let (selected, on_select) = match selection {
        Some((at, on_select)) => {
            let found = rows.clone();
            let on_select = Callback::new(move |key: Option<RowKey>| {
                on_select.call(key.and_then(|key| found.at(key)));
            });
            (at.filter(|&at| at < rows.len()).map(|at| rows.key(at)), Some(on_select))
        }
        None => (None, None),
    };
    table::body(table::Body {
        source: rows,
        widths: laid,
        look: painted_look(look, palette),
        selected,
        on_select,
    })
}
