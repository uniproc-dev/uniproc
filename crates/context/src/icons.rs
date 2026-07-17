pub use guicons::{IconData, IconFamily, IconKey, IconRef, IconVariant};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/icons.rs"));
}

pub use generated::*;

// TODO: this shim only exists to keep the old `Icons::resolve`/`get_key`
// call sites unchanged after switching to guicons, which generates free
// functions (`resolve`) instead of a struct. Drop it by either upstreaming
// a `guicons`-side `Icons`-style API or, more likely, updating the callers
// (slint-adapter) to call `resolve()`/`get_key` directly.
pub struct Icons;

impl Icons {
    pub fn get_key(key: IconKey) -> slint::Image {
        resolve_image(key)
    }

    pub fn resolve<'a>(icon: impl Into<IconRef<'a>>) -> slint::Image {
        resolve_image(icon)
    }
}

fn resolve_image<'a>(icon: impl Into<IconRef<'a>>) -> slint::Image {
    // Icons materialize as `IconData::Svg` (local files/iconify/url sources
    // are all SVG); `Png`/`Glyph` have no Rust-side image representation
    // here - glyphs render via the Slint `Icon` component's `Text` branch
    // instead, not through this resolver.
    match generated::resolve(icon) {
        Some(IconData::Svg(bytes)) => slint::Image::load_from_svg_data(bytes).unwrap_or_default(),
        _ => slint::Image::default(),
    }
}
