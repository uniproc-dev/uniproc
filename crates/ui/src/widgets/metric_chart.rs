use guinea::prelude::Load;
use guinea_widgets::chart::{Chart, HoverInfo, Interpolation, LineChartOptions, Series};
use guinea_widgets::color::hex;
use windows_canvas::ColorF;
use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength,
    HorizontalAlignment, LayoutControl, Orientation, StackPanel, ThemeBrush, Thickness,
    VerticalAlignment, View,
};

use crate::l10n::tr;
use crate::theme::{radius, space, Palette};
use crate::widgets::text::caption;

#[derive(Clone, Copy, Debug)]
pub enum MetricChartKind {
    Cpu,
    Memory,
}

impl MetricChartKind {
    pub fn title(&self) -> String {
        match self {
            Self::Cpu => tr().metric_chart_cpu(),
            Self::Memory => tr().metric_chart_memory(),
        }
    }

    pub fn color(&self) -> ColorF {
        match self {
            Self::Cpu => hex(0x60a5fa),
            Self::Memory => hex(0x34d399),
        }
    }

    fn color_direct(&self) -> Color {
        match self {
            Self::Cpu => Color::rgb(0x60, 0xa5, 0xfa),
            Self::Memory => Color::rgb(0x34, 0xd3, 0x99),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MetricChartStyle {
    pub show_grid: bool,
    pub card: bool,
}

#[expect(non_upper_case_globals)]
impl MetricChartStyle {
    pub const Card: Self = Self { show_grid: true, card: true };
    pub const Sparkline: Self = Self { show_grid: false, card: false };
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

pub fn metric_mini_bar(
    palette: Palette,
    kind: MetricChartKind,
    history: &Load<Vec<(u64, f32)>>,
) -> View {
    let current = history
        .ready()
        .and_then(|points| points.last())
        .map(|&(_, v)| v)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);

    let fill_color = if current >= MiniBar::CriticalFillFrom {
        MiniBar::Critical
    } else {
        kind.color_direct()
    };
    let track_color = if current > MiniBar::CriticalAbove {
        MiniBar::TrackCritical
    } else if current > MiniBar::WarningAbove {
        MiniBar::TrackWarning
    } else {
        palette.track_idle
    };

    let fill = Border::new()
        .height(MiniBar::Height * (current as f64 / 100.0))
        .background(fill_color)
        .corner_radius(MiniBar::Width / 2.0)
        .vertical_alignment(VerticalAlignment::Bottom)
        .content(View::empty());

    Border::new()
        .width(MiniBar::Width)
        .height(MiniBar::Height)
        .corner_radius(MiniBar::Width / 2.0)
        .background(track_color)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Bottom)
        .content(fill)
}

pub struct MetricChart<'a> {
    pub chart: &'a Chart,
    pub kind: MetricChartKind,
    pub history: &'a Load<Vec<(u64, f32)>>,
    pub height: f64,
    pub style: MetricChartStyle,
    pub detail: Option<String>,
    pub palette: Palette,
}

pub fn metric_chart(props: MetricChart<'_>) -> View {
    let MetricChart {
        chart,
        kind,
        history,
        height,
        style,
        detail,
        palette,
    } = props;

    let points = history.ready().cloned().unwrap_or_default();
    let current = points.last().map(|&(_, v)| v).unwrap_or(0.0);

    chart.publish(
        vec![Series {
            color: kind.color(),
            interpolation: Interpolation::Linear,
            fill: None,
            points,
        }],
        LineChartOptions {
            background: None,
            border: None,
            show_grid: style.show_grid,
            y_range: Some((0.0, 100.0)),
        },
    );
    let surface = Border::new()
        .height(height)
        .content(chart.view(|_: Option<HoverInfo>| {}));

    let label: View = match detail {
        Some(detail) => StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Control)
            .children((
                caption(kind.title()),
                caption(detail).foreground(palette.tertiary_text),
            )),
        None => caption(kind.title()).into(),
    };
    let header = Grid::new()
        .columns([GridLength::Auto, GridLength::Star(1.0)])
        .children((
            Border::new().grid_column(0).content(label),
            caption(format!("{current:.1}%"))
                .foreground(palette.secondary_text)
                .horizontal_alignment(HorizontalAlignment::Right)
                .grid_column(1),
        ));

    let content = StackPanel::new()
        .spacing(space::Compact)
        .children((header, surface));

    if style.card {
        Border::new()
            .background(ThemeBrush::CardBackground)
            .border_brush(ThemeBrush::CardStroke)
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(radius::Overlay)
            .padding(Thickness::uniform(space::Control))
            .content(content)
    } else {
        content
    }
}
