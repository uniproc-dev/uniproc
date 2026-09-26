use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows_reactor::{Color, ColorScheme};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Palette {
    pub heat_muted: Color,
    pub track_idle: Color,
    pub secondary_text: Color,
    pub tertiary_text: Color,
    pub disabled_text: Color,
    pub row_selected: Color,
    pub row_hovered: Color,
    pub divider_stroke: Color,
    pub layer_fill: Color,
    pub success: Color,
    pub caution: Color,
    pub menu_fill: Color,
    pub menu_stroke: Color,
    pub menu_shadow: Color,
}

impl Palette {
    pub fn of(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Dark => Self {
                heat_muted: Color::argb(255, 150, 150, 150),
                track_idle: Color::argb(26, 255, 255, 255),
                secondary_text: Color::argb(197, 255, 255, 255),
                tertiary_text: Color::argb(139, 255, 255, 255),
                disabled_text: Color::argb(93, 255, 255, 255),
                row_selected: Color::argb(21, 255, 255, 255),
                row_hovered: Color::argb(10, 255, 255, 255),
                divider_stroke: Color::argb(21, 255, 255, 255),
                layer_fill: Color::argb(76, 58, 58, 58),
                success: Color::argb(255, 0x6C, 0xCB, 0x5F),
                caution: Color::argb(255, 0xFC, 0xE1, 0x00),
                menu_fill: Color::argb(255, 0x2C, 0x2C, 0x2C),
                menu_stroke: Color::argb(26, 255, 255, 255),
                menu_shadow: Color::argb(72, 0, 0, 0),
            },
            ColorScheme::Light => Self {
                heat_muted: Color::argb(255, 110, 110, 110),
                track_idle: Color::argb(26, 0, 0, 0),
                secondary_text: Color::argb(158, 0, 0, 0),
                tertiary_text: Color::argb(114, 0, 0, 0),
                disabled_text: Color::argb(92, 0, 0, 0),
                row_selected: Color::argb(15, 0, 0, 0),
                row_hovered: Color::argb(8, 0, 0, 0),
                divider_stroke: Color::argb(15, 0, 0, 0),
                layer_fill: Color::argb(128, 255, 255, 255),
                success: Color::argb(255, 0x0F, 0x7B, 0x0F),
                caution: Color::argb(255, 0x9D, 0x5D, 0x00),
                menu_fill: Color::argb(255, 0xF9, 0xF9, 0xF9),
                menu_stroke: Color::argb(15, 0, 0, 0),
                menu_shadow: Color::argb(28, 0, 0, 0),
            },
        }
    }
}

struct Accent;

#[expect(non_upper_case_globals)]
impl Accent {
    const Fallback: Color = Color {
        a: 255,
        r: 0,
        g: 120,
        b: 212,
    };
}

pub fn accent_color() -> Color {
    UISettings::new()
        .and_then(|s| s.GetColorValue(UIColorType::Accent))
        .map(|c| Color {
            a: c.A,
            r: c.R,
            g: c.G,
            b: c.B,
        })
        .unwrap_or(Accent::Fallback)
}
