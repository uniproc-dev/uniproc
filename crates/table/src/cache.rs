use std::collections::HashMap;
use std::hash::Hash;

pub struct Cache<K, V> {
    entries: HashMap<K, (V, u64)>,
    now: u64,
    limit: usize,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(limit: usize) -> Self {
        Self { entries: HashMap::new(), now: 0, limit }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.now += 1;
        let now = self.now;
        self.entries.get_mut(key).map(|(value, used)| {
            *used = now;
            &*value
        })
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.now += 1;
        self.entries.insert(key, (value, self.now));
    }

    pub fn sweep(&mut self) {
        let excess = self.entries.len().saturating_sub(self.limit);
        if excess == 0 {
            return;
        }
        let mut stamps: Vec<u64> = self.entries.values().map(|(_, used)| *used).collect();
        stamps.sort_unstable();
        let oldest_kept = stamps[excess];
        self.entries.retain(|_, (_, used)| *used >= oldest_kept);
    }
}

pub fn premultiplied_bgra(rgba: &[u8]) -> Vec<u8> {
    let scale = |channel: u8, alpha: u8| ((channel as u32 * alpha as u32 + 127) / 255) as u8;
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| [scale(b, a), scale(g, a), scale(r, a), a])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_the_limit_nothing_is_swept() {
        let mut cache = Cache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");
        cache.sweep();
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
    }

    #[test]
    fn over_the_limit_the_least_recently_used_go_first() {
        let mut cache = Cache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");
        cache.insert(3, "c");
        assert_eq!(cache.get(&1), Some(&"a"));
        cache.sweep();
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&3), Some(&"c"));
    }

    #[test]
    fn straight_rgba_becomes_premultiplied_bgra() {
        assert_eq!(premultiplied_bgra(&[255, 128, 0, 255, 200, 100, 50, 128, 9, 9, 9, 0]), [0, 128, 255, 255, 25, 50, 100, 128, 0, 0, 0, 0]);
    }
}
