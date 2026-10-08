#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Whole,
    Cut(usize),
}

pub fn trim(advances: &[f32], room: f32, ellipsis: f32) -> Fit {
    if advances.iter().sum::<f32>() <= room {
        return Fit::Whole;
    }
    let room = room - ellipsis;
    let mut used = 0.0;
    let kept = advances
        .iter()
        .take_while(|advance| {
            used += **advance;
            used <= room
        })
        .count();
    Fit::Cut(kept)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_that_fits_is_left_whole() {
        assert_eq!(trim(&[10.0, 10.0, 10.0], 30.0, 8.0), Fit::Whole);
    }

    #[test]
    fn text_that_does_not_fit_keeps_what_fits_beside_the_ellipsis() {
        assert_eq!(trim(&[10.0, 10.0, 10.0, 10.0], 35.0, 8.0), Fit::Cut(2));
    }

    #[test]
    fn no_room_even_for_one_glyph_keeps_none() {
        assert_eq!(trim(&[10.0, 10.0], 12.0, 8.0), Fit::Cut(0));
    }

    #[test]
    fn empty_text_is_whole() {
        assert_eq!(trim(&[], 0.0, 8.0), Fit::Whole);
    }
}
