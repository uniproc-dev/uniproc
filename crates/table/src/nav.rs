use crate::layout::Lines;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

pub fn step(lines: &Lines, current: Option<usize>, step: Step, viewport: f32) -> Option<usize> {
    let last = lines.len().checked_sub(1)?;
    let Some(at) = current.map(|at| at.min(last)) else {
        return Some(if step == Step::End { last } else { 0 });
    };
    Some(match step {
        Step::Up => at.saturating_sub(1),
        Step::Down => (at + 1).min(last),
        Step::Home => 0,
        Step::End => last,
        Step::PageDown => {
            let bottom = lines.start(at) + viewport;
            let fits = lines.at(bottom).map_or(last, |below| below.saturating_sub(1));
            fits.max(at + 1).min(last)
        }
        Step::PageUp => {
            let top = lines.start(at) + lines.size(at) - viewport;
            let fits = match lines.at(top.max(0.0)) {
                Some(above) if lines.start(above) < top => above + 1,
                Some(above) => above,
                None => 0,
            };
            fits.min(at.saturating_sub(1))
        }
    })
}

pub fn reveal(lines: &Lines, at: usize, offset: f32, viewport: f32) -> f32 {
    let top = lines.start(at);
    let bottom = top + lines.size(at);
    if top < offset {
        top
    } else if bottom > offset + viewport {
        bottom - viewport
    } else {
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(count: usize) -> Lines {
        Lines::new(std::iter::repeat_n(32.0, count))
    }

    #[test]
    fn nothing_to_step_through_selects_nothing() {
        assert_eq!(step(&rows(0), None, Step::Down, 320.0), None);
    }

    #[test]
    fn with_nothing_selected_a_step_starts_at_an_end() {
        let lines = rows(50);
        assert_eq!(step(&lines, None, Step::Down, 320.0), Some(0));
        assert_eq!(step(&lines, None, Step::Up, 320.0), Some(0));
        assert_eq!(step(&lines, None, Step::End, 320.0), Some(49));
    }

    #[test]
    fn arrows_move_one_row_and_stop_at_the_ends() {
        let lines = rows(50);
        assert_eq!(step(&lines, Some(5), Step::Down, 320.0), Some(6));
        assert_eq!(step(&lines, Some(5), Step::Up, 320.0), Some(4));
        assert_eq!(step(&lines, Some(0), Step::Up, 320.0), Some(0));
        assert_eq!(step(&lines, Some(49), Step::Down, 320.0), Some(49));
    }

    #[test]
    fn home_and_end_go_to_the_first_and_last_row() {
        let lines = rows(50);
        assert_eq!(step(&lines, Some(20), Step::Home, 320.0), Some(0));
        assert_eq!(step(&lines, Some(20), Step::End, 320.0), Some(49));
    }

    #[test]
    fn a_page_moves_to_the_last_row_that_fits_below_and_back() {
        let lines = rows(50);
        assert_eq!(step(&lines, Some(0), Step::PageDown, 320.0), Some(9));
        assert_eq!(step(&lines, Some(9), Step::PageDown, 320.0), Some(18));
        assert_eq!(step(&lines, Some(18), Step::PageUp, 320.0), Some(9));
        assert_eq!(step(&lines, Some(45), Step::PageDown, 320.0), Some(49));
        assert_eq!(step(&lines, Some(4), Step::PageUp, 320.0), Some(0));
    }

    #[test]
    fn a_page_shorter_than_a_row_still_moves_one() {
        let lines = rows(50);
        assert_eq!(step(&lines, Some(3), Step::PageDown, 20.0), Some(4));
        assert_eq!(step(&lines, Some(3), Step::PageUp, 20.0), Some(2));
    }

    #[test]
    fn revealing_a_visible_row_keeps_the_offset() {
        assert_eq!(reveal(&rows(50), 12, 320.0, 320.0), 320.0);
    }

    #[test]
    fn revealing_a_row_above_scrolls_it_to_the_top() {
        assert_eq!(reveal(&rows(50), 5, 320.0, 320.0), 160.0);
    }

    #[test]
    fn revealing_a_row_below_scrolls_it_to_the_bottom() {
        assert_eq!(reveal(&rows(50), 25, 320.0, 320.0), 512.0);
    }
}
