mod appx;
mod bitmap;
mod exe;
#[cfg(test)]
mod goldens;
mod trim;
mod window;

pub use appx::extract_appx_icon_rgba;
pub use exe::{extract_icon_rgba, has_own_icon};
pub use window::extract_window_icon_rgba;
