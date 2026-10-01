use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, Grid, HorizontalAlignment, LayoutControl,
    TextBlock, TextTrimming, TextWrapping, Thickness, VerticalAlignment, View,
};

use guicons::icon;

use crate::theme::{opacity, radius, size, space, Palette};
use crate::widgets::text::caption;

#[derive(Clone, Copy)]
pub struct Heat {
    pub share: f32,
    pub threshold: f32,
    pub color: Color,
}

#[expect(non_upper_case_globals)]
impl Heat {
    pub const Threshold: f32 = 0.007;
    const Ceiling: f32 = 230.0;
    const Curve: f32 = 0.5;
}

impl Heat {
    pub fn alpha(self) -> u8 {
        let clamped = self.share.clamp(0.0, 1.0);
        if clamped < self.threshold {
            return 0;
        }
        (clamped.powf(Self::Curve) * Self::Ceiling) as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Highlight {
    pub top: bool,
    pub bottom: bool,
}

#[expect(non_upper_case_globals)]
impl Highlight {
    pub const Whole: Self = Self {
        top: true,
        bottom: true,
    };
}

pub fn cell_text(content: impl Into<String>) -> TextBlock {
    caption(content)
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
}

pub fn dimmed(content: impl Into<View>, dim: bool) -> View {
    let content = content.into();
    if dim {
        Border::new().opacity(opacity::Stopped).content(content).into()
    } else {
        content
    }
}

pub fn service_icon() -> View {
    icon!(gears).size(size::Icon).build_element()
}

fn text_cell(content: impl Into<String>) -> TextBlock {
    cell_text(content)
        .height(size::TableRow)
        .max_height(size::TableRow)
}

pub fn heat_cell(content: impl Into<String>, intensity: f32, accent: Color) -> View {
    let heat = Heat {
        share: intensity,
        threshold: Heat::Threshold,
        color: accent,
    };
    let wash = Color {
        a: heat.alpha(),
        ..accent
    };
    Border::new()
        .background(wash)
        .corner_radius(radius::Control)
        .margin(Thickness::xy(space::Compact, 0.0))
        .content(text_cell(content))
}

pub struct Metric {
    pub text: String,
    pub zero: bool,
    pub heat: Option<Heat>,
    pub height: f64,
}

pub fn metric_cell(metric: Metric, palette: Palette) -> View {
    let value = cell_text(metric.text)
        .horizontal_alignment(HorizontalAlignment::Right)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::xy(space::Cell, 0.0));
    let value = if metric.zero {
        value.foreground(palette.disabled_text)
    } else {
        value
    };

    let wash: View = match metric.heat {
        Some(heat) if heat.alpha() > 0 => Border::new()
            .background(Color {
                a: heat.alpha(),
                ..heat.color
            })
            .corner_radius(radius::Control)
            .margin(Thickness::uniform(space::Compact))
            .into(),
        _ => View::empty(),
    };

    Grid::new()
        .height(metric.height)
        .children((wash, value))
        .into()
}
