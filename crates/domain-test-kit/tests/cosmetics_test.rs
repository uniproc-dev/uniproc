use domain::features::cosmetics::CosmeticsFeature;
use domain_test_kit::test_env::cosmetics::CosmeticsPortStub;
use domain_test_kit::utils::{DomainTestWindow, FeatureHarness, temp_settings_path};
use forsl::settings::SettingsFeature;
use i_slint_core::api::ComponentHandle;
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
    let stub = CosmeticsPortStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        CosmeticsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    let ui_handle = h.0.as_ref().unwrap().ui().clone_strong();
    h.0.as_mut()
        .unwrap()
        .spawn_window(ui_handle)
        .expect("Failed to spawn window");

    assert!(
        stub.apply_main_window_effects_called(),
        "ApplyMainWindowEffects MUST be sent when the feature installs"
    );
}
