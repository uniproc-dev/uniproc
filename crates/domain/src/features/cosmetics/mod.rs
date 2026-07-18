use app_contracts::features::cosmetics::{UiCosmeticsPort, UiCosmeticsPortMsg};
use forsl::app::Window;
use forsl::feature::{FromWindow, IntoWindowFeature, WindowFeature, WindowFeatureInitContext};
use macros::window_feature;

#[derive(Clone, Copy, Debug)]
pub struct AccentState(pub forsl::native_windows::platform_types::AccentPalette);

#[window_feature]
pub fn cosmetics_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiCosmeticsPort + Clone + 'static,
{
    if let Ok(accent_palette) = forsl::native_windows::platform::get_system_accent_palette() {
        port.send(UiCosmeticsPortMsg::SetAccentPalette(
            app_contracts::features::cosmetics::AccentPalette {
                accent: accent_palette.accent.into(),
                accent_light_1: accent_palette.accent_light_1.into(),
                accent_light_2: accent_palette.accent_light_2.into(),
                accent_light_3: accent_palette.accent_light_3.into(),
                accent_dark_1: accent_palette.accent_dark_1.into(),
                accent_dark_2: accent_palette.accent_dark_2.into(),
                accent_dark_3: accent_palette.accent_dark_3.into(),
            },
        ));
        ctx.shared.insert(AccentState(accent_palette));
    }
    port.send(UiCosmeticsPortMsg::ApplyMainWindowEffects);
    Ok(())
}
