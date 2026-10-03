use app_contracts::features::activity::Histogram;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength, HorizontalAlignment,
    LayoutControl, PointerEventInfo, Rectangle, Thickness, VerticalAlignment, View,
};

use super::super::marks::ActivityMark;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, space, Palette};
use crate::widgets::text::caption;

struct Bars;

#[expect(non_upper_case_globals)]
impl Bars {
    const Half: f64 = 28.0;
    const Least: f64 = 3.0;
    const Gap: f64 = 1.0;
    const Axis: f64 = 1.0;
}

fn height(count: u32, most: u32) -> f64 {
    if count == 0 {
        return 0.0;
    }
    (f64::from(count) / f64::from(most) * Bars::Half).max(Bars::Least)
}

fn bucket(at: usize, came: f64, went: f64, picked: bool, palette: Palette, on_pick: &Callback<usize>) -> View {
    let on_pick = on_pick.clone();
    Border::new()
        .grid_column(at as i32)
        .background(if picked { palette.row_selected } else { Color::transparent() })
        .padding(Thickness::xy(Bars::Gap / 2.0, 0.0))
        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
            let _ = on_pick.call(at);
        }))
        .content(
            Grid::new()
                .rows([
                    GridLength::Pixel(Bars::Half),
                    GridLength::Pixel(Bars::Axis),
                    GridLength::Pixel(Bars::Half),
                ])
                .children((
                    Rectangle::new()
                        .grid_row(0)
                        .height(came)
                        .vertical_alignment(VerticalAlignment::Bottom)
                        .fill(accent_color()),
                    Rectangle::new().grid_row(1).fill(palette.divider_stroke),
                    Rectangle::new()
                        .grid_row(2)
                        .height(went)
                        .vertical_alignment(VerticalAlignment::Top)
                        .fill(palette.secondary_text),
                )),
        )
}

pub fn histogram(histogram: &Histogram, l10n: &L10n, palette: Palette, on_pick: Callback<usize>) -> View {
    let most = histogram
        .buckets
        .iter()
        .map(|bucket| bucket.came.max(bucket.went))
        .max()
        .unwrap_or(0)
        .max(1);
    let picked = |at: usize| histogram.picked.is_some_and(|(from, to)| (from..to).contains(&at));
    let bars: Vec<(String, View)> = histogram
        .buckets
        .iter()
        .enumerate()
        .map(|(at, counts)| {
            (
                at.to_string(),
                bucket(at, height(counts.came, most), height(counts.went, most), picked(at), palette, &on_pick),
            )
        })
        .collect();

    let axis = Grid::new()
        .grid_column(0)
        .grid_row(0)
        .rows([GridLength::Star(1.0), GridLength::Star(1.0)])
        .margin(Thickness::new(0.0, 0.0, space::Control, 0.0))
        .children((
            caption(l10n.activity_axis_came())
                .grid_row(0)
                .horizontal_alignment(HorizontalAlignment::Right)
                .vertical_alignment(VerticalAlignment::Center)
                .foreground(palette.secondary_text),
            caption(l10n.activity_axis_went())
                .grid_row(1)
                .horizontal_alignment(HorizontalAlignment::Right)
                .vertical_alignment(VerticalAlignment::Center)
                .foreground(palette.secondary_text),
        ));

    let columns = vec![GridLength::Star(1.0); histogram.buckets.len().max(1)];
    let plot = Grid::new()
        .mark(ActivityMark::Histogram)
        .grid_column(1)
        .grid_row(0)
        .columns(columns)
        .children((View::keyed_fragment(bars),));

    let last = histogram.ticks.len().saturating_sub(1);
    let ticks: Vec<(String, View)> = histogram
        .ticks
        .iter()
        .enumerate()
        .map(|(at, tick)| {
            let column = at.min(last.saturating_sub(1)) as i32;
            let alignment = if at == last && at > 0 {
                HorizontalAlignment::Right
            } else {
                HorizontalAlignment::Left
            };
            (
                at.to_string(),
                caption(format::clock(*tick))
                    .grid_column(column)
                    .horizontal_alignment(alignment)
                    .foreground(palette.secondary_text)
                    .into(),
            )
        })
        .collect();
    let scale = Grid::new()
        .grid_column(1)
        .grid_row(1)
        .columns(vec![GridLength::Star(1.0); last.max(1)])
        .children((View::keyed_fragment(ticks),));

    Grid::new()
        .columns([GridLength::Auto, GridLength::Star(1.0)])
        .rows([GridLength::Auto, GridLength::Auto])
        .children((axis, plot, scale))
}
