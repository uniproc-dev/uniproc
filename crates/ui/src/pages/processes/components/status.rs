use guinea::winui::MarkExt;
use windows_reactor::{
    Border, ChildrenControl, ContentControl, HorizontalAlignment, KeyedView, LayoutControl, Orientation,
    StackPanel, Thickness, View,
};

use super::super::marks::ProcessesMark;
use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::text::text;

struct Count;

#[expect(non_upper_case_globals)]
impl Count {
    const Digit: f64 = 8.0;
    const Total: f64 = Self::Digit * 4.0;
    const Part: f64 = Self::Digit * 3.0;
    const Gap: f64 = 4.0;
}

pub(crate) struct StatusCounts {
    pub(crate) total: usize,
    pub(crate) apps: usize,
    pub(crate) background: usize,
    pub(crate) services: usize,
    pub(crate) kernel: usize,
    pub(crate) linux: usize,
    pub(crate) pinned: usize,
}

fn segment(count: usize, slot: f64, label: String, palette: Palette) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .children((
            Border::new()
                .mark(ProcessesMark::StatusCount)
                .min_width(slot)
                .content(
                    text(count.to_string())
                        .foreground(palette.secondary_text)
                        .horizontal_alignment(HorizontalAlignment::Right),
                ),
            text(label)
                .foreground(palette.secondary_text)
                .margin(Thickness::new(Count::Gap, 0.0, 0.0, 0.0)),
        ))
        .into()
}

fn dot(palette: Palette) -> View {
    text("·")
        .foreground(palette.tertiary_text)
        .margin(Thickness::xy(space::Control, 0.0))
        .into()
}

pub(crate) fn status_bar(counts: &StatusCounts, l10n: &L10n, palette: Palette) -> View {
    let count = |n: usize| n as i64;
    let mut segments = vec![
        segment(counts.total, Count::Total, l10n.processes_status_processes(count(counts.total)), palette),
        segment(counts.apps, Count::Part, l10n.processes_status_apps(count(counts.apps)), palette),
        segment(counts.background, Count::Part, l10n.processes_status_background(), palette),
        segment(counts.services, Count::Part, l10n.processes_status_services(count(counts.services)), palette),
        segment(counts.kernel, Count::Part, l10n.processes_status_kernel(), palette),
    ];
    if counts.linux > 0 {
        segments.push(segment(counts.linux, Count::Part, l10n.processes_status_wsl(), palette));
    }
    if counts.pinned > 0 {
        segments.push(segment(counts.pinned, Count::Part, l10n.processes_status_pinned(), palette));
    }

    let mut line = Vec::with_capacity(segments.len() * 2);
    for (at, segment) in segments.into_iter().enumerate() {
        if at > 0 {
            line.push(dot(palette));
        }
        line.push(segment);
    }
    let line = View::keyed_fragment(line.into_iter().enumerate().map(|(at, view)| KeyedView::new(at, view)));
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .children((line,))
        .into()
}
