use crate::features::cosmetics::UiCosmeticsAdapter;
use app_contracts::features::cosmetics::{UiCosmeticsPort, UiCosmeticsPortMsg};

use forsl::native_windows::{NativeWindowConfig, apply_to_component};
use forsl_macros::port_adapter;
use slint::ComponentHandle;

#[port_adapter(backend = "slint", window = AppWindow)]
impl UiCosmeticsPort for UiCosmeticsAdapter {
    fn send(&self, ui: &AppWindow, msg: UiCosmeticsPortMsg) {
        match msg {
            UiCosmeticsPortMsg::ApplyMainWindowEffects => {
                #[cfg(target_os = "windows")]
                {
                    apply_to_component(ui.as_weak(), NativeWindowConfig::win11_dialog());
                }
            }
            UiCosmeticsPortMsg::SetAccentPalette(palette) => {
                let theme = ui.global::<crate::Theme>();
                theme.set_accent(palette.accent);
                theme.set_accent_light_1(palette.accent_light_1);
                theme.set_accent_light_2(palette.accent_light_2);
                theme.set_accent_light_3(palette.accent_light_3);
                theme.set_accent_dark_1(palette.accent_dark_1);
                theme.set_accent_dark_2(palette.accent_dark_2);
                theme.set_accent_dark_3(palette.accent_dark_3);
            }
        }
    }
}
