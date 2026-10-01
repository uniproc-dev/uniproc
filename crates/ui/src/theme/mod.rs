pub mod opacity;
pub mod palette;
pub mod radius;
pub mod setting;
pub mod size;
pub mod space;

use windows_reactor::{ColorScheme, Context};

pub use palette::{accent_color, Palette};

pub fn scheme_context() -> &'static Context<ColorScheme> {
    thread_local! {
        static SCHEME: &'static Context<ColorScheme> =
            Box::leak(Box::new(Context::new(ColorScheme::default())));
    }
    SCHEME.with(|scheme| *scheme)
}
