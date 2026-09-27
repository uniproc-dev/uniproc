use std::cmp::Ordering;

use super::grouping::{DisplayRow, DropEdge, SectionId};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SectionGesture {
    Grab { section: SectionId, at: f64, offset: f64 },
    Move { at: f64 },
    Release,
    Lost,
}

struct Drag;

#[expect(non_upper_case_globals)]
impl Drag {
    const Threshold: f64 = 4.0;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Grab {
    pub(crate) section: SectionId,
    from: f64,
    at: f64,
    offset: f64,
    held: bool,
}

impl Grab {
    pub(crate) fn new(section: SectionId, at: f64, offset: f64) -> Self {
        Self {
            section,
            from: at,
            at,
            offset,
            held: true,
        }
    }

    pub(crate) fn follow(&mut self, at: f64) {
        if self.held {
            self.at = at;
        }
    }

    pub(crate) fn let_go(&mut self) {
        self.held = false;
    }

    pub(crate) fn moved(&self) -> bool {
        (self.at - self.from).abs() >= Drag::Threshold
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Placement {
    pub(crate) before: Option<SectionId>,
}

struct Span {
    id: SectionId,
    top: f64,
    bottom: f64,
}

fn spans(rows: &[DisplayRow]) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut y = 0.0;
    for d in rows {
        let bottom = y + d.height();
        match (&d.section, spans.last_mut()) {
            (Some(section), _) => spans.push(Span {
                id: section.id,
                top: y,
                bottom,
            }),
            (None, Some(span)) => span.bottom = bottom,
            (None, None) => {}
        }
        y = bottom;
    }
    spans
}

pub(crate) fn placement(rows: &[DisplayRow], grab: &Grab) -> Option<Placement> {
    if !grab.moved() {
        return None;
    }
    let spans = spans(rows);
    let from = spans.iter().position(|span| span.id == grab.section)?;
    let pointer = spans[from].top + grab.offset + (grab.at - grab.from);
    let over = spans
        .iter()
        .position(|span| pointer < span.bottom)
        .unwrap_or(spans.len() - 1);
    match over.cmp(&from) {
        Ordering::Equal => None,
        Ordering::Less => Some(Placement {
            before: Some(spans[over].id),
        }),
        Ordering::Greater => Some(Placement {
            before: spans.get(over + 1).map(|span| span.id),
        }),
    }
}

pub(crate) fn mark(rows: &mut [DisplayRow], grab: &Grab, placement: Option<Placement>) {
    if !grab.moved() || !grab.held {
        return;
    }
    let heading = |d: &DisplayRow, id: SectionId| d.section.as_ref().is_some_and(|section| section.id == id);
    for d in rows.iter_mut() {
        d.lifted = heading(d, grab.section);
    }
    let edge = match placement {
        Some(Placement { before: Some(id) }) => rows
            .iter_mut()
            .find(|d| heading(d, id))
            .map(|d| (d, DropEdge::Above)),
        Some(Placement { before: None }) => rows.last_mut().map(|d| (d, DropEdge::Below)),
        None => None,
    };
    if let Some((d, edge)) = edge {
        d.drop_edge = Some(edge);
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::processes::ProcessCategory;

    use super::*;
    use crate::pages::processes::components::grouping::tests::{heading, process};
    use crate::theme::size;

    const APPS: SectionId = SectionId::Category(ProcessCategory::App);
    const THIRD_PARTY: SectionId = SectionId::Category(ProcessCategory::BackgroundThirdParty);
    const KERNEL: SectionId = SectionId::Category(ProcessCategory::WindowsKernel);

    fn table() -> Vec<DisplayRow> {
        vec![
            heading(APPS),
            process(1),
            process(2),
            heading(THIRD_PARTY),
            process(3),
            heading(KERNEL),
            process(4),
        ]
    }

    fn dragged(section: SectionId, by: f64) -> Grab {
        let mut grab = Grab::new(section, 100.0, size::SectionRow / 2.0);
        grab.follow(100.0 + by);
        grab
    }

    #[test]
    fn a_nudge_is_not_a_drag() {
        assert_eq!(placement(&table(), &dragged(APPS, 3.0)), None);
        assert_eq!(placement(&table(), &dragged(APPS, 20.0)), None, "still over its own section");
    }

    #[test]
    fn down_into_the_next_section_goes_after_it() {
        let to_third_party_heading = 2.0 * size::ProcessRow + size::SectionRow / 2.0;
        assert_eq!(
            placement(&table(), &dragged(APPS, to_third_party_heading)),
            Some(Placement { before: Some(KERNEL) })
        );
        assert_eq!(
            placement(&table(), &dragged(APPS, 1_000.0)),
            Some(Placement { before: None }),
            "past the end is the end"
        );
    }

    #[test]
    fn up_into_the_previous_section_goes_before_it() {
        assert_eq!(
            placement(&table(), &dragged(KERNEL, -size::ProcessRow)),
            Some(Placement { before: Some(THIRD_PARTY) })
        );
        assert_eq!(
            placement(&table(), &dragged(KERNEL, -1_000.0)),
            Some(Placement { before: Some(APPS) }),
            "above the top is the top"
        );
    }

    #[test]
    fn the_lifted_heading_and_the_gap_are_marked() {
        let mut rows = table();
        let grab = dragged(APPS, 1_000.0);
        let place = placement(&rows, &grab);
        mark(&mut rows, &grab, place);
        let lifted: Vec<usize> = (0..rows.len()).filter(|at| rows[*at].lifted).collect();
        let edges: Vec<(usize, DropEdge)> =
            rows.iter().enumerate().filter_map(|(at, d)| d.drop_edge.map(|edge| (at, edge))).collect();
        assert_eq!(lifted, [0]);
        assert_eq!(edges, [(6, DropEdge::Below)]);

        let mut rows = table();
        let grab = dragged(KERNEL, -size::ProcessRow);
        let place = placement(&rows, &grab);
        mark(&mut rows, &grab, place);
        let edges: Vec<(usize, DropEdge)> =
            rows.iter().enumerate().filter_map(|(at, d)| d.drop_edge.map(|edge| (at, edge))).collect();
        assert_eq!(edges, [(3, DropEdge::Above)]);
    }
}
