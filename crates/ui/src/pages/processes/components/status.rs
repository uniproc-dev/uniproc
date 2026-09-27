use guinea::winui::MarkExt;
use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, CornerRadius, Grid, GridChildExt, GridLength,
    HorizontalAlignment, KeyedView, LayoutControl, Orientation, StackPanel, Thickness, VerticalAlignment,
    View,
};

use super::super::marks::ProcessesMark;
use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::text::text;

struct Share;

#[expect(non_upper_case_globals)]
impl Share {
    const Height: f64 = 4.0;
    const Radius: f64 = Self::Height / 2.0;
    const Swatch: f64 = 8.0;
    const SwatchRadius: f64 = 2.0;
    const Digit: f64 = 8.0;
    const Total: f64 = Self::Digit * 4.0;
    const Part: f64 = Self::Digit * 3.0;
    const Gap: f64 = 4.0;
    const Between: f64 = 14.0;
}

pub(crate) struct StatusCounts {
    pub(crate) apps: usize,
    pub(crate) background: usize,
    pub(crate) services: usize,
    pub(crate) kernel: usize,
    pub(crate) linux: usize,
}

impl StatusCounts {
    fn total(&self) -> usize {
        self.apps + self.background + self.services + self.kernel + self.linux
    }
}

struct Part {
    count: usize,
    color: Color,
    label: String,
}

fn parts(counts: &StatusCounts, l10n: &L10n, palette: Palette) -> Vec<Part> {
    let part = |count: usize, color, label| Part { count, color, label };
    let mut parts = vec![
        part(counts.apps, palette.share_apps, l10n.processes_status_apps(counts.apps as i64)),
        part(counts.background, palette.share_background, l10n.processes_status_background()),
        part(counts.services, palette.share_services, l10n.processes_status_services(counts.services as i64)),
        part(counts.kernel, palette.share_kernel, l10n.processes_status_kernel()),
    ];
    if counts.linux > 0 {
        parts.push(part(counts.linux, palette.share_wsl, l10n.processes_status_wsl()));
    }
    parts
}

fn keyed(views: Vec<View>) -> View {
    View::keyed_fragment(views.into_iter().enumerate().map(|(at, view)| KeyedView::new(at, view)))
}

fn share_bar(parts: &[Part]) -> View {
    let shown: Vec<&Part> = parts.iter().filter(|part| part.count > 0).collect();
    let last = shown.len().saturating_sub(1);
    let segments = shown
        .iter()
        .enumerate()
        .map(|(at, part)| {
            let left = if at == 0 { Share::Radius } else { 0.0 };
            let right = if at == last { Share::Radius } else { 0.0 };
            Border::new()
                .mark(ProcessesMark::StatusShare)
                .grid_column(at as i32)
                .background(part.color)
                .corner_radius(CornerRadius::new(left, right, right, left))
                .into()
        })
        .collect();
    Grid::new()
        .height(Share::Height)
        .margin(Thickness::new(0.0, 0.0, 0.0, space::Compact))
        .columns(shown.iter().map(|part| GridLength::Star(part.count as f64)))
        .children((keyed(segments),))
        .into()
}

fn count(n: usize, slot: f64, palette: Palette) -> View {
    Border::new()
        .mark(ProcessesMark::StatusCount)
        .min_width(slot)
        .content(
            text(n.to_string())
                .foreground(palette.secondary_text)
                .horizontal_alignment(HorizontalAlignment::Right),
        )
        .into()
}

fn label(label: String, palette: Palette) -> View {
    text(label)
        .foreground(palette.secondary_text)
        .margin(Thickness::new(Share::Gap, 0.0, 0.0, 0.0))
        .into()
}

fn legend_item(part: &Part, palette: Palette) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .children((
            Border::new()
                .width(Share::Swatch)
                .height(Share::Swatch)
                .corner_radius(Share::SwatchRadius)
                .background(part.color)
                .vertical_alignment(VerticalAlignment::Center),
            count(part.count, Share::Part, palette),
            label(part.label.clone(), palette),
        ))
        .into()
}

pub(crate) fn status_bar(counts: &StatusCounts, l10n: &L10n, palette: Palette) -> View {
    let parts = parts(counts, l10n, palette);
    let total = counts.total();
    let legend = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(Share::Between)
        .grid_column(0)
        .children((keyed(parts.iter().map(|part| legend_item(part, palette)).collect()),));
    let total = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .grid_column(2)
        .children((
            count(total, Share::Total, palette),
            label(l10n.processes_status_processes(total as i64), palette),
        ));

    StackPanel::new()
        .children((
            share_bar(&parts),
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .children((legend, total)),
        ))
        .into()
}
