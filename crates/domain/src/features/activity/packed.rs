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
}
