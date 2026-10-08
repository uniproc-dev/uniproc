use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lines {
    edges: Vec<f32>,
}

impl Lines {
    pub fn new(sizes: impl IntoIterator<Item = f32>) -> Self {
        let mut edges = vec![0.0];
        let mut end = 0.0;
        for size in sizes {
            end += size;
            edges.push(end);
        }
        Self { edges }
    }

    pub fn len(&self) -> usize {
        self.edges.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn extent(&self) -> f32 {
        self.edges[self.len()]
    }

    pub fn start(&self, at: usize) -> f32 {
        self.edges[at]
    }

    pub fn size(&self, at: usize) -> f32 {
        self.edges[at + 1] - self.edges[at]
    }

    pub fn at(&self, offset: f32) -> Option<usize> {
        if offset < 0.0 || offset >= self.extent() {
            return None;
        }
        Some(self.edges.partition_point(|&edge| edge <= offset) - 1)
    }

    pub fn span(&self, from: f32, to: f32) -> Range<usize> {
        let first = self.edges[1..].partition_point(|&end| end <= from);
        let last = self.edges[..self.len()].partition_point(|&start| start < to);
        first..last.max(first)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub below: f32,
    pub above: f32,
}

impl Band {
    pub const ALWAYS: Band = Band {
        below: f32::INFINITY,
        above: f32::NEG_INFINITY,
    };

    pub fn holds(self, offset: f32) -> bool {
        self.below <= offset && offset <= self.above
    }
}

pub fn realized(lines: &Lines, offset: f32, viewport: f32, overscan: f32) -> Range<usize> {
    lines.span(offset - overscan, offset + viewport + overscan)
}

pub fn band(lines: &Lines, realized: Range<usize>, viewport: f32, margin: f32) -> Band {
    let below = if realized.start == 0 {
        f32::NEG_INFINITY
    } else {
        lines.start(realized.start) + margin
    };
    let above = if realized.end >= lines.len() {
        f32::INFINITY
    } else {
        lines.start(realized.end) - viewport - margin
    };
    Band { below, above }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Lines {
        Lines::new([36.0, 32.0, 32.0, 36.0, 32.0])
    }

    #[test]
    fn lines_of_different_sizes_stack() {
        let lines = lines();
        assert_eq!(lines.len(), 5);
        assert_eq!(lines.extent(), 168.0);
        assert_eq!(lines.start(0), 0.0);
        assert_eq!(lines.start(3), 100.0);
        assert_eq!(lines.size(3), 36.0);
    }

    #[test]
    fn an_offset_finds_the_line_under_it() {
        let lines = lines();
        assert_eq!(lines.at(0.0), Some(0));
        assert_eq!(lines.at(35.9), Some(0));
        assert_eq!(lines.at(36.0), Some(1));
        assert_eq!(lines.at(167.0), Some(4));
        assert_eq!(lines.at(168.0), None);
        assert_eq!(lines.at(-1.0), None);
    }

    #[test]
    fn a_span_holds_every_line_it_touches() {
        let lines = lines();
        assert_eq!(lines.span(40.0, 101.0), 1..4);
        assert_eq!(lines.span(36.0, 68.0), 1..2);
        assert_eq!(lines.span(-50.0, 10.0), 0..1);
        assert_eq!(lines.span(150.0, 900.0), 4..5);
        assert_eq!(lines.span(500.0, 900.0), 5..5);
    }

    #[test]
    fn realized_lines_reach_past_the_viewport_by_the_overscan() {
        let lines = Lines::new(std::iter::repeat_n(32.0, 100));
        assert_eq!(realized(&lines, 320.0, 320.0, 64.0), 8..22);
        assert_eq!(realized(&lines, 0.0, 320.0, 64.0), 0..12);
        assert_eq!(realized(&lines, 2880.0, 320.0, 64.0), 88..100);
    }

    #[test]
    fn the_band_wakes_before_the_viewport_leaves_what_was_drawn() {
        let lines = Lines::new(std::iter::repeat_n(32.0, 100));
        let band = band(&lines, 8..22, 320.0, 32.0);
        assert_eq!(band, Band { below: 288.0, above: 352.0 });
        assert!(band.holds(320.0));
        assert!(!band.holds(287.0));
        assert!(!band.holds(353.0));
    }

    #[test]
    fn the_band_has_no_edge_where_the_lines_end() {
        let lines = Lines::new(std::iter::repeat_n(32.0, 100));
        let top = band(&lines, 0..12, 320.0, 32.0);
        assert!(top.holds(-100.0));
        assert!(!top.holds(33.0));
        let bottom = band(&lines, 88..100, 320.0, 32.0);
        assert!(bottom.holds(5000.0));
    }
}
