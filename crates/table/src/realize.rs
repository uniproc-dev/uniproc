use std::collections::HashMap;
use std::ops::Range;

use crate::model::{Cell, RowKey, Source};

pub trait Visuals {
    type Visual;

    fn create(&mut self) -> Self::Visual;
    fn retire(&mut self, visual: &Self::Visual);
    fn revive(&mut self, visual: &Self::Visual);
    fn place(&mut self, visual: &Self::Visual, at: usize);
    fn draw(&mut self, key: RowKey, visual: &Self::Visual, cells: &[Cell], changed: &[bool]);
}

pub struct Row<V> {
    pub visual: V,
    pub at: usize,
    pub cells: Vec<Cell>,
    seen: u64,
}

pub struct Realized<V> {
    rows: HashMap<RowKey, Row<V>>,
    pool: Vec<V>,
    pass: u64,
    gone: Vec<RowKey>,
    changed: Vec<bool>,
    scratch: Cell,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pass {
    pub created: usize,
    pub reused: usize,
    pub retired: usize,
    pub moved: usize,
    pub drawn: usize,
}

impl<V> Default for Realized<V> {
    fn default() -> Self {
        Self {
            rows: HashMap::new(),
            pool: Vec::new(),
            pass: 0,
            gone: Vec::new(),
            changed: Vec::new(),
            scratch: Cell::default(),
        }
    }
}

impl<V> Realized<V> {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn pooled(&self) -> usize {
        self.pool.len()
    }

    pub fn get(&self, key: RowKey) -> Option<&Row<V>> {
        self.rows.get(&key)
    }

    pub fn rows(&self) -> impl Iterator<Item = (RowKey, &Row<V>)> {
        self.rows.iter().map(|(key, row)| (*key, row))
    }

    pub fn update(&mut self, source: &dyn Source, range: Range<usize>, visuals: &mut impl Visuals<Visual = V>) -> Pass {
        let mut pass = Pass::default();
        let range = range.start.min(source.len())..range.end.min(source.len());

        self.pass = self.pass.wrapping_add(1);
        for at in range.clone() {
            if let Some(row) = self.rows.get_mut(&source.key(at)) {
                row.seen = self.pass;
            }
        }
        self.gone.clear();
        self.gone.extend(self.rows.iter().filter(|(_, row)| row.seen != self.pass).map(|(key, _)| *key));
        for key in self.gone.drain(..) {
            if let Some(row) = self.rows.remove(&key) {
                visuals.retire(&row.visual);
                self.pool.push(row.visual);
                pass.retired += 1;
            }
        }

        let columns = source.columns();
        for at in range {
            let key = source.key(at);
            let fresh = !self.rows.contains_key(&key);
            if fresh {
                let visual = match self.pool.pop() {
                    Some(visual) => {
                        visuals.revive(&visual);
                        pass.reused += 1;
                        visual
                    }
                    None => {
                        pass.created += 1;
                        visuals.create()
                    }
                };
                self.rows.insert(key, Row { visual, at: usize::MAX, cells: Vec::new(), seen: self.pass });
            }
            let Some(row) = self.rows.get_mut(&key) else {
                continue;
            };
            if row.at != at {
                if !fresh {
                    pass.moved += 1;
                }
                row.at = at;
                visuals.place(&row.visual, at);
            }
            let fresh = fresh || row.cells.len() != columns;
            row.cells.resize_with(columns, Cell::default);
            self.changed.clear();
            self.changed.resize(columns, fresh);
            let mut any = fresh;
            for column in 0..columns {
                source.cell(at, column, &mut self.scratch);
                if self.scratch != row.cells[column] {
                    std::mem::swap(&mut self.scratch, &mut row.cells[column]);
                    self.changed[column] = true;
                    any = true;
                }
            }
            if any {
                visuals.draw(key, &row.visual, &row.cells, &self.changed);
                pass.drawn += 1;
            }
        }
        pass
    }

    pub fn forget(&mut self) {
        for row in self.rows.values_mut() {
            row.cells.clear();
        }
    }

    pub fn clear(&mut self, mut drop: impl FnMut(V)) {
        for (_, row) in self.rows.drain() {
            drop(row.visual);
        }
        for visual in self.pool.drain(..) {
            drop(visual);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        rows: Vec<(RowKey, [&'static str; 2])>,
    }

    impl Source for Fake {
        fn len(&self) -> usize {
            self.rows.len()
        }

        fn key(&self, at: usize) -> RowKey {
            self.rows[at].0
        }

        fn height(&self, _at: usize) -> f32 {
            32.0
        }

        fn columns(&self) -> usize {
            2
        }

        fn cell(&self, at: usize, column: usize, out: &mut Cell) {
            out.clear();
            out.text.push_str(self.rows[at].1[column]);
        }
    }

    fn fake(rows: &[(RowKey, [&'static str; 2])]) -> Fake {
        Fake { rows: rows.to_vec() }
    }

    #[derive(Default)]
    struct Recorder {
        made: u32,
        drawn: Vec<(RowKey, Vec<bool>, Vec<String>)>,
        placed: Vec<(u32, usize)>,
    }

    impl Visuals for Recorder {
        type Visual = u32;

        fn create(&mut self) -> u32 {
            self.made += 1;
            self.made
        }

        fn retire(&mut self, _visual: &u32) {}

        fn revive(&mut self, _visual: &u32) {}

        fn place(&mut self, visual: &u32, at: usize) {
            self.placed.push((*visual, at));
        }

        fn draw(&mut self, key: RowKey, _visual: &u32, cells: &[Cell], changed: &[bool]) {
            let texts = cells.iter().map(|cell| cell.text.clone()).collect();
            self.drawn.push((key, changed.to_vec(), texts));
        }
    }

    fn pass(realized: &mut Realized<u32>, source: &Fake, range: Range<usize>, recorder: &mut Recorder) -> Pass {
        recorder.drawn.clear();
        recorder.placed.clear();
        realized.update(source, range, recorder)
    }

    #[test]
    fn the_first_pass_creates_places_and_draws_every_row_in_range() {
        let source = fake(&[(1, ["a", "1"]), (2, ["b", "2"]), (3, ["c", "3"])]);
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        let pass = pass(&mut realized, &source, 0..2, &mut recorder);
        assert_eq!(pass, Pass { created: 2, drawn: 2, ..Pass::default() });
        assert_eq!(recorder.placed, [(1, 0), (2, 1)]);
        assert_eq!(
            recorder.drawn,
            [
                (1, vec![true, true], vec!["a".to_string(), "1".to_string()]),
                (2, vec![true, true], vec!["b".to_string(), "2".to_string()]),
            ]
        );
        assert_eq!(realized.get(2).map(|row| row.at), Some(1));
    }

    #[test]
    fn nothing_changed_draws_nothing() {
        let source = fake(&[(1, ["a", "1"]), (2, ["b", "2"])]);
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        pass(&mut realized, &source, 0..2, &mut recorder);
        let again = pass(&mut realized, &source, 0..2, &mut recorder);
        assert_eq!(again, Pass::default());
        assert!(recorder.drawn.is_empty() && recorder.placed.is_empty());
    }

    #[test]
    fn a_resort_moves_rows_without_drawing_them() {
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        pass(&mut realized, &fake(&[(1, ["a", "1"]), (2, ["b", "2"])]), 0..2, &mut recorder);
        let visual = realized.get(1).map(|row| row.visual);
        let resorted = pass(&mut realized, &fake(&[(2, ["b", "2"]), (1, ["a", "1"])]), 0..2, &mut recorder);
        assert_eq!(resorted, Pass { moved: 2, ..Pass::default() });
        assert!(recorder.drawn.is_empty());
        assert_eq!(realized.get(1).map(|row| row.visual), visual);
        assert_eq!(realized.get(1).map(|row| row.at), Some(1));
    }

    #[test]
    fn only_the_cells_that_changed_are_marked() {
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        pass(&mut realized, &fake(&[(1, ["a", "1"])]), 0..1, &mut recorder);
        let changed = pass(&mut realized, &fake(&[(1, ["a", "9"])]), 0..1, &mut recorder);
        assert_eq!(changed, Pass { drawn: 1, ..Pass::default() });
        assert_eq!(recorder.drawn, [(1, vec![false, true], vec!["a".to_string(), "9".to_string()])]);
    }

    #[test]
    fn forgotten_rows_are_drawn_whole_again_on_their_visuals() {
        let source = fake(&[(1, ["a", "1"]), (2, ["b", "2"])]);
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        pass(&mut realized, &source, 0..2, &mut recorder);
        realized.forget();
        let again = pass(&mut realized, &source, 0..2, &mut recorder);
        assert_eq!(again, Pass { drawn: 2, ..Pass::default() });
        assert_eq!(recorder.made, 2);
        assert!(recorder.drawn.iter().all(|(_, changed, _)| changed.iter().all(|c| *c)));
    }

    #[test]
    fn rows_leaving_the_range_wait_in_the_pool_and_come_back_for_new_rows() {
        let source = fake(&[(1, ["a", "1"]), (2, ["b", "2"]), (3, ["c", "3"]), (4, ["d", "4"])]);
        let mut realized = Realized::default();
        let mut recorder = Recorder::default();
        pass(&mut realized, &source, 0..2, &mut recorder);
        let scrolled = pass(&mut realized, &source, 2..4, &mut recorder);
        assert_eq!(scrolled, Pass { reused: 2, retired: 2, drawn: 2, ..Pass::default() });
        assert_eq!(recorder.made, 2);
        assert!(realized.get(1).is_none());
        assert_eq!(realized.get(4).map(|row| row.at), Some(3));
        assert!(recorder.drawn.iter().all(|(_, changed, _)| changed.iter().all(|c| *c)));
    }
}
