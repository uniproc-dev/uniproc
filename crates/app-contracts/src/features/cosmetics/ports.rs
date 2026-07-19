use forsl_macros::port;

use super::model::AccentPalette;

#[derive(Clone, Copy, Debug)]
pub enum UiCosmeticsPortMsg {
    ApplyMainWindowEffects,
    SetAccentPalette(AccentPalette),
}

#[port]
pub trait UiCosmeticsPort: Clone + 'static {
    fn send(&self, msg: UiCosmeticsPortMsg);
}
