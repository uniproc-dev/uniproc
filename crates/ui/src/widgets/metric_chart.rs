use app_contracts::features::settings::{ByteUnits, SidebarChart};
use guinea::prelude::Load;
use guinea_widgets::chart::{Chart, ChartGrid, HoverInfo, Interpolation, LineChartOptions, Series};
use guinea_widgets::color::{hex, hex_alpha};
use windows_canvas::ColorF;
use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength, HorizontalAlignment,
    LayoutControl, StackPanel, ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::format;
use crate::l10n::tr;
use crate::theme::{radius, space, Palette};
use crate::widgets::text::caption;

pub fn chart_title(chart: SidebarChart) -> String {
    match chart {
        SidebarChart::Cpu => tr().metric_chart_cpu(),
        SidebarChart::Memory => tr().metric_chart_memory(),
        SidebarChart::Disk => tr().metric_chart_disk(),
        SidebarChart::Network => tr().metric_chart_network(),
        SidebarChart::Gpu => tr().metric_chart_gpu(),
    }
}

fn rgb(chart: SidebarChart) -> u32 {
    match chart {
        SidebarChart::Cpu => 0x60a5fa,
        SidebarChart::Memory => 0x34d399,
        SidebarChart::Disk => 0xfbbf24,
        SidebarChart::Network => 0xf472b6,
        SidebarChart::Gpu => 0xa78bfa,
    }
}

fn line_color(chart: SidebarChart) -> ColorF {
    hex(rgb(chart))
}

fn fill_color(chart: SidebarChart) -> ColorF {
    const FILL_ALPHA: u8 = 36;
    hex_alpha(rgb(chart), FILL_ALPHA)
}

fn bar_color(chart: SidebarChart) -> Color {
    let rgb = rgb(chart);
    Color::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

#[derive(Clone, Copy, Debug)]
pub enum Scale {
    Percent,
    Rate(ByteUnits),
}

impl Scale {
    pub fn warns(self) -> bool {
        matches!(self, Scale::Percent)
    }
}

struct Timeline;

#[expect(non_upper_case_globals)]
impl Timeline {
    const Tick: u64 = 5_000;
    const Window: u64 = 60_000;
    const Levels: [f32; 3] = [25.0, 50.0, 75.0];
}

struct RateScale;

#[expect(non_upper_case_globals)]
impl RateScale {
    const Least: u64 = 100 << 10;
    const Units: [u64; 4] = [1 << 10, 1 << 20, 1 << 30, 1 << 40];
    const Steps: [u64; 9] = [1, 2, 5, 10, 20, 50, 100, 200, 500];
}

pub fn rate_ceiling(points: &[(u64, f32)]) -> u64 {
    let latest = points.last().map_or(0, |&(t, _)| t);
    let peak = points
        .iter()
        .filter(|&&(t, _)| latest.saturating_sub(t) <= Timeline::Window)
        .map(|&(_, v)| v.max(0.0) as u64)
        .max()
        .unwrap_or(0);
    RateScale::Units
        .iter()
        .flat_map(|unit| RateScale::Steps.iter().map(move |step| step * unit))
        .filter(|ceiling| *ceiling >= RateScale::Least)
        .find(|ceiling| *ceiling >= peak)
        .unwrap_or(peak)
}

fn current(history: &Load<Vec<(u64, f32)>>) -> f32 {
    history.ready().and_then(|points| points.last()).map_or(0.0, |&(_, v)| v)
}

pub fn chart_level(history: &Load<Vec<(u64, f32)>>, scale: Scale) -> f32 {
    let value = current(history);
    match scale {
        Scale::Percent => value,
        Scale::Rate(_) => {
            let ceiling = history.ready().map_or(0, |points| rate_ceiling(points));
            if ceiling == 0 {
                0.0
            } else {
                value / ceiling as f32 * 100.0
            }
        }
    }
}

struct MiniBar;

#[expect(non_upper_case_globals)]
impl MiniBar {
    const Width: f64 = 6.0;
    const Height: f64 = 34.0;

    const WarningAbove: f32 = 65.0;
    const CriticalAbove: f32 = 90.0;
    const CriticalFillFrom: f32 = 95.0;

    const Warning: Color = Color { a: 255, r: 0xFF, g: 0xB9, b: 0x00 };
    const Critical: Color = Color { a: 255, r: 0xE7, g: 0x48, b: 0x56 };
    const TrackWarning: Color = Color { a: 130, ..Self::Warning };
    const TrackCritical: Color = Color { a: 130, ..Self::Critical };
}

pub fn metric_mini_bar(palette: Palette, chart: SidebarChart, level: f32, warns: bool) -> View {
    let level = level.clamp(0.0, 100.0);

    let fill = if warns && level >= MiniBar::CriticalFillFrom {
        MiniBar::Critical
    } else {
        bar_color(chart)
    };
    let track = if warns && level > MiniBar::CriticalAbove {
        MiniBar::TrackCritical
    } else if warns && level > MiniBar::WarningAbove {
        MiniBar::TrackWarning
    } else {
        palette.track_idle
    };

    let bar = Border::new()
        .height(MiniBar::Height * (level as f64 / 100.0))
        .background(fill)
        .corner_radius(MiniBar::Width / 2.0)
        .vertical_alignment(VerticalAlignment::Bottom)
        .content(View::empty());

    Border::new()
        .width(MiniBar::Width)
        .height(MiniBar::Height)
        .corner_radius(MiniBar::Width / 2.0)
        .background(track)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Bottom)
        .content(bar)
}

fn color_f(color: Color) -> ColorF {
    ColorF::from_rgba8(color.r, color.g, color.b, color.a)
}

pub struct MetricChart<'a> {
    pub chart: &'a Chart,
    pub kind: SidebarChart,
    pub history: &'a Load<Vec<(u64, f32)>>,
    pub scale: Scale,
    pub height: f64,
    pub detail: Option<String>,
    pub palette: Palette,
}

pub fn metric_chart(props: MetricChart<'_>) -> View {
    let MetricChart {
        chart,
        kind,
        history,
        scale,
        height,
        detail,
        palette,
    } = props;

    let points = history.ready().cloned().unwrap_or_default();
    let now = points.last().map_or(0.0, |&(_, v)| v);
    let (ceiling, reading, bound) = match scale {
        Scale::Percent => (100.0, format!("{now:.1}%"), None),
        Scale::Rate(units) => {
            let ceiling = rate_ceiling(&points);
            (
                ceiling as f32,
                format::bytes_per_second(units, now.max(0.0) as u64),
                Some(tr().metric_chart_scale(format::rate_bound(units, ceiling))),
            )
        }
    };

    chart.publish(
        vec![Series {
            color: line_color(kind),
            interpolation: Interpolation::Linear,
            fill: Some(fill_color(kind)),
            points,
        }],
        LineChartOptions {
            background: None,
            border: None,
            grid: Some(ChartGrid {
                every_t: Some(Timeline::Tick),
                at_v: Timeline::Levels.iter().map(|level| ceiling * level / 100.0).collect(),
                color: color_f(palette.divider_stroke),
            }),
            x_window: Some(Timeline::Window),
            y_range: Some((0.0, ceiling.max(1.0))),
            corner_radius: Some((radius::Control - 1.0) as f32),
        },
    );

    let reading = match detail {
        Some(detail) => format!("{reading} · {detail}"),
        None => reading,
    };
    let title = Grid::new()
        .columns([GridLength::Auto, GridLength::Star(1.0)])
        .children((
            caption(chart_title(kind)).grid_column(0),
            match bound {
                Some(bound) => caption(bound)
                    .foreground(palette.tertiary_text)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .grid_column(1)
                    .into(),
                None => View::empty(),
            },
        ));
    let label = StackPanel::new()
        .margin(Thickness::xy(space::Header, space::Compact))
        .vertical_alignment(VerticalAlignment::Top)
        .children((title, caption(reading).foreground(palette.secondary_text)));

    Border::new()
        .height(height)
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(radius::Control)
        .content(Grid::new().children((chart.view(|_: Option<HoverInfo>| {}), label)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rate_scale_rounds_the_peak_up_to_a_step() {
        assert_eq!(rate_ceiling(&[]), 100 << 10);
        assert_eq!(rate_ceiling(&[(0, 3_000.0)]), 100 << 10);
        assert_eq!(rate_ceiling(&[(0, 150.0 * 1024.0)]), 200 << 10);
        assert_eq!(rate_ceiling(&[(0, 60.0 * 1024.0 * 1024.0)]), 100 << 20);
        assert_eq!(rate_ceiling(&[(1_000, 3.0 * 1024.0 * 1024.0)]), 5 << 20);
    }

    #[test]
    fn a_peak_that_left_the_window_no_longer_holds_the_scale() {
        let points = [(0, 90.0 * 1024.0 * 1024.0), (Timeline::Window + 1, 1_000.0)];
        assert_eq!(rate_ceiling(&points), 100 << 10);
    }
}
