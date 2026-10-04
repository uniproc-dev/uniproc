use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows_canvas::ColorF;
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
    pub critical: Color,
    pub menu_fill: Color,
    pub menu_stroke: Color,
    pub menu_shadow: Color,
    pub card_fill: Color,
    pub card_fill_hovered: Color,
    pub card_fill_pressed: Color,
    pub card_stroke: Color,
    pub share_apps: Color,
    pub share_background: Color,
    pub share_services: Color,
    pub share_kernel: Color,
    pub share_wsl: Color,
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
                critical: Color::argb(255, 0xFF, 0x99, 0xA4),
                menu_fill: Color::argb(255, 0x2C, 0x2C, 0x2C),
                menu_stroke: Color::argb(26, 255, 255, 255),
                menu_shadow: Color::argb(72, 0, 0, 0),
                card_fill: Color::argb(13, 255, 255, 255),
                card_fill_hovered: Color::argb(21, 255, 255, 255),
                card_fill_pressed: Color::argb(8, 255, 255, 255),
                card_stroke: Color::argb(25, 0, 0, 0),
                share_apps: Color::argb(255, 0x4C, 0xC2, 0xFF),
                share_background: Color::argb(255, 0x6C, 0xCB, 0x5F),
                share_services: Color::argb(255, 0xB4, 0xA0, 0xFF),
                share_kernel: Color::argb(255, 0x9E, 0x9E, 0x9E),
                share_wsl: Color::argb(255, 0xFF, 0x9A, 0x5C),
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
                critical: Color::argb(255, 0xC4, 0x2B, 0x1C),
                menu_fill: Color::argb(255, 0xF9, 0xF9, 0xF9),
                menu_stroke: Color::argb(15, 0, 0, 0),
                menu_shadow: Color::argb(28, 0, 0, 0),
                card_fill: Color::argb(179, 255, 255, 255),
                card_fill_hovered: Color::argb(128, 249, 249, 249),
                card_fill_pressed: Color::argb(77, 249, 249, 249),
                card_stroke: Color::argb(15, 0, 0, 0),
                share_apps: Color::argb(255, 0x00, 0x5F, 0xB8),
                share_background: Color::argb(255, 0x0F, 0x7B, 0x0F),
                share_services: Color::argb(255, 0x6B, 0x4F, 0xBB),
                share_kernel: Color::argb(255, 0x6E, 0x6E, 0x6E),
                share_wsl: Color::argb(255, 0xC2, 0x4E, 0x00),
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

pub fn color_f(color: Color) -> ColorF {
    ColorF::from_rgba8(color.r, color.g, color.b, color.a)
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
