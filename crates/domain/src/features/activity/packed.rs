use std::sync::Arc;

pub struct Pool<T> {
    spare: Vec<Arc<Vec<T>>>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self { spare: Vec::new() }
    }
}

struct Spare;

#[expect(non_upper_case_globals)]
impl Spare {
    const Kept: usize = 64;
}

impl<T> Pool<T> {
    pub fn retire(&mut self, items: Arc<Vec<T>>) {
        if self.spare.len() < Spare::Kept {
            self.spare.push(items);
        }
    }

    pub fn take(&mut self) -> Arc<Vec<T>> {
        for at in 0..self.spare.len() {
            if let Some(items) = Arc::get_mut(&mut self.spare[at]) {
                items.clear();
                return self.spare.swap_remove(at);
            }
        }
        Arc::default()
    }
}

#[derive(Default)]
pub struct Numbers {
    bytes: Vec<u8>,
}

impl Numbers {
    pub fn put<const N: usize>(&mut self, values: [u64; N]) -> u32 {
        let at = self.bytes.len() as u32;
        for mut value in values {
            while value >= 0x80 {
                self.bytes.push(value as u8 | 0x80);
                value >>= 7;
            }
            self.bytes.push(value as u8);
        }
        at
    }

    pub fn get<const N: usize>(&self, at: u32) -> [u64; N] {
        let mut values = [0; N];
        let mut bytes = self.bytes[at as usize..].iter();
        for value in &mut values {
            let mut shift = 0;
            for &byte in bytes.by_ref() {
                *value |= u64::from(byte & 0x7f) << shift;
                shift += 7;
                if byte & 0x80 == 0 {
                    break;
                }
            }
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_back_as_they_were_put_small_ones_in_a_byte() {
        let mut numbers = Numbers::default();
        let small = numbers.put([0, 3, 127]);
        let large = numbers.put([128, 4_123_456_789, u64::MAX, 1 << 35]);

        assert_eq!(numbers.get::<3>(small), [0, 3, 127]);
        assert_eq!(numbers.get::<4>(large), [128, 4_123_456_789, u64::MAX, 1 << 35]);
        assert_eq!(large, 3);
    }

    #[test]
    fn a_buffer_nobody_holds_any_more_is_handed_out_again_empty() {
        let mut pool = Pool::default();
        let buffer = Arc::new(vec![7u64; 50]);
        let kept = buffer.as_ptr();
        pool.retire(buffer);

        let taken = pool.take();

        assert_eq!((taken.as_ptr(), taken.len(), taken.capacity() >= 50), (kept, 0, true));
    }

    #[test]
    fn a_buffer_still_held_elsewhere_waits_until_it_is_let_go() {
        let mut pool = Pool::default();
        let buffer = Arc::new(vec![7u64; 50]);
        let held = buffer.clone();
        pool.retire(buffer);

        let meanwhile = pool.take();
        assert_ne!(meanwhile.as_ptr(), held.as_ptr());

        let kept = held.as_ptr();
        drop(held);
        assert_eq!(pool.take().as_ptr(), kept);
    }
}
