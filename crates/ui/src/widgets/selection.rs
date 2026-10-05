#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectionMark {
    Keeper,
}

pub fn keep_place<T, K: PartialEq>(
    items: &mut Vec<T>,
    key: K,
    holds: impl Fn(&T) -> bool,
    previous: Option<(K, usize)>,
) -> Option<(K, usize)> {
    let current = items.iter().position(holds)?;
    let target = previous
        .filter(|(pinned, _)| *pinned == key)
        .map_or(current, |(_, at)| at.min(items.len() - 1));
    if target != current {
        let item = items.remove(current);
        items.insert(target, item);
    }
    Some((key, target))
}

pub struct Pinned<T, K> {
    place: Option<(K, usize)>,
    held: Option<T>,
}

impl<T, K> Default for Pinned<T, K> {
    fn default() -> Self {
        Self { place: None, held: None }
    }
}

impl<T: Clone, K: PartialEq + Copy> Pinned<T, K> {
    pub fn place(&mut self, items: &[T], selected: Option<K>, key: impl Fn(&T) -> K) -> Vec<T> {
        let mut placed = items.to_vec();
        let Some(selected) = selected else {
            *self = Self::default();
            return placed;
        };
        if self.place.is_some_and(|(pinned, _)| pinned != selected) {
            *self = Self::default();
        }
        let holds = |item: &T| key(item) == selected;
        if !placed.iter().any(holds)
            && let Some(held) = self.held.clone()
        {
            let at = self.place.map_or(0, |(_, at)| at).min(placed.len());
            placed.insert(at, held);
        }
        self.place = keep_place(&mut placed, selected, holds, self.place);
        self.held = placed.iter().find(|item| holds(item)).cloned();
        placed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Item = (u32, &'static str);

    fn keys(items: &[Item]) -> Vec<u32> {
        items.iter().map(|item| item.0).collect()
    }

    fn place(pinned: &mut Pinned<Item, u32>, items: &[Item], selected: Option<u32>) -> Vec<u32> {
        keys(&pinned.place(items, selected, |item| item.0))
    }

    #[test]
    fn a_selected_item_keeps_its_place_when_newer_items_come_on_top() {
        let mut pinned = Pinned::default();
        assert_eq!(place(&mut pinned, &[(1, "a"), (2, "b"), (3, "c")], Some(2)), [1, 2, 3]);

        let placed = place(&mut pinned, &[(9, "x"), (8, "y"), (1, "a"), (2, "b"), (3, "c")], Some(2));

        assert_eq!(placed, [9, 2, 8, 1, 3]);
    }

    #[test]
    fn a_selected_item_that_leaves_stays_where_it_was() {
        let mut pinned = Pinned::default();
        place(&mut pinned, &[(1, "a"), (2, "b"), (3, "c")], Some(2));

        assert_eq!(place(&mut pinned, &[(1, "a"), (3, "c")], Some(2)), [1, 2, 3]);
        assert_eq!(place(&mut pinned, &[(1, "a"), (3, "c")], Some(2)), [1, 2, 3]);
    }

    #[test]
    fn selecting_another_item_lets_the_held_one_go() {
        let mut pinned = Pinned::default();
        place(&mut pinned, &[(1, "a"), (2, "b"), (3, "c")], Some(2));
        place(&mut pinned, &[(1, "a"), (3, "c")], Some(2));

        assert_eq!(place(&mut pinned, &[(1, "a"), (3, "c")], Some(3)), [1, 3]);
        assert_eq!(place(&mut pinned, &[(1, "a"), (3, "c")], None), [1, 3]);
    }

    #[test]
    fn a_held_item_shows_as_it_was_last_seen() {
        let mut pinned = Pinned::default();
        place(&mut pinned, &[(1, "a"), (2, "b")], Some(2));
        place(&mut pinned, &[(1, "a"), (2, "b, exited")], Some(2));

        let placed = pinned.place(&[(1, "a")], Some(2), |item| item.0);

        assert_eq!(placed, [(1, "a"), (2, "b, exited")]);
    }
}
