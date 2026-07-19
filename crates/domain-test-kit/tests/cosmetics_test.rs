use app_contracts::features::cosmetics::UiCosmeticsPortMsg;
use domain::features::cosmetics::CosmeticsFeature;
use domain_test_kit::generated::CosmeticsUiStub;
use domain_test_kit::utils::{DomainTestWindow, FeatureHarness, temp_settings_path};
use forsl::settings::SettingsFeature;
use rstest::{fixture, rstest};
use serial_test::serial;

#[fixture]
fn h() -> FeatureHarness {
    let temp_path = temp_settings_path();

    FeatureHarness::new(temp_path.clone())
        .app_feature(SettingsFeature::with_path(temp_path))
        .unwrap()
}

#[rstest]
#[serial]
fn test_cosmetics_feature_applies_main_window_effects_on_install(mut h: FeatureHarness) {
    let stub = CosmeticsUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        CosmeticsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");

    let messages = stub.ui_cosmetics_port_sent().stabilize(&mut h);
    assert!(
        messages
            .iter()
            .any(|msg| matches!(msg, UiCosmeticsPortMsg::ApplyMainWindowEffects)),
        "ApplyMainWindowEffects MUST be sent when the feature installs"
    );
}
