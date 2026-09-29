use guinea::prelude::Load;
use guinea_widgets::chart::{Chart, ChartGrid, HoverInfo, Interpolation, LineChartOptions, Series};
use guinea_widgets::color::{hex, hex_alpha};
use windows_canvas::ColorF;
use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, Grid, HorizontalAlignment, LayoutControl,
    StackPanel, ThemeBrush, Thickness, VerticalAlignment, View,
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

    fn fill(&self) -> ColorF {
        const FILL_ALPHA: u8 = 36;
        match self {
            Self::Cpu => hex_alpha(0x60a5fa, FILL_ALPHA),
            Self::Memory => hex_alpha(0x34d399, FILL_ALPHA),
        }
    }

    fn color_direct(&self) -> Color {
        match self {
            Self::Cpu => Color::rgb(0x60, 0xa5, 0xfa),
            Self::Memory => Color::rgb(0x34, 0xd3, 0x99),
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

struct Timeline;

#[expect(non_upper_case_globals)]
impl Timeline {
    const Tick: u64 = 5_000;
    const Window: u64 = 60_000;
    const Half: f32 = 50.0;
}

fn color_f(color: Color) -> ColorF {
    ColorF::from_rgba8(color.r, color.g, color.b, color.a)
}

pub struct MetricChart<'a> {
    pub chart: &'a Chart,
    pub kind: MetricChartKind,
    pub history: &'a Load<Vec<(u64, f32)>>,
    pub height: f64,
    pub detail: Option<String>,
    pub palette: Palette,
}

pub fn metric_chart(props: MetricChart<'_>) -> View {
    let MetricChart {
        chart,
        kind,
        history,
        height,
        detail,
        palette,
    } = props;

    let points = history.ready().cloned().unwrap_or_default();
    let current = points.last().map(|&(_, v)| v).unwrap_or(0.0);

    chart.publish(
        vec![Series {
            color: kind.color(),
            interpolation: Interpolation::Linear,
            fill: Some(kind.fill()),
            points,
        }],
        LineChartOptions {
            background: None,
            border: None,
            grid: Some(ChartGrid {
                every_t: Some(Timeline::Tick),
                at_v: vec![Timeline::Half],
                color: color_f(palette.divider_stroke),
            }),
            x_window: Some(Timeline::Window),
            y_range: Some((0.0, 100.0)),
        },
    );

    let value = match detail {
        Some(detail) => format!("{current:.1}% · {detail}"),
        None => format!("{current:.1}%"),
    };
    let label = StackPanel::new()
        .margin(Thickness::xy(space::Header, space::Compact))
        .vertical_alignment(VerticalAlignment::Top)
        .children((
            caption(kind.title()),
            caption(value).foreground(palette.secondary_text),
        ));

    Border::new()
        .height(height)
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(radius::Control)
        .content(Grid::new().children((chart.view(|_: Option<HoverInfo>| {}), label)))
}
