use windows_reactor::{FontWeight, TextBlock};

struct FontSize;

#[expect(non_upper_case_globals)]
impl FontSize {
    const Caption: f64 = 12.0;
    const Body: f64 = 14.0;
    const NavLabel: f64 = 15.0;
    const BodyLarge: f64 = 18.0;
    const Subtitle: f64 = 20.0;
}

pub fn text(content: impl Into<String>) -> TextBlock {
    TextBlock::new().text(content)
}

pub fn caption(content: impl Into<String>) -> TextBlock {
    text(content).font_size(FontSize::Caption)
}

pub fn body_strong(content: impl Into<String>) -> TextBlock {
    text(content)
        .font_size(FontSize::Body)
        .font_weight(FontWeight::SEMI_BOLD)
}

pub fn nav_label(content: impl Into<String>) -> TextBlock {
    text(content).font_size(FontSize::NavLabel)
}

pub fn body_large(content: impl Into<String>) -> TextBlock {
    text(content).font_size(FontSize::BodyLarge)
}

pub fn subtitle(content: impl Into<String>) -> TextBlock {
    text(content)
        .font_size(FontSize::Subtitle)
        .font_weight(FontWeight::SEMI_BOLD)
}
