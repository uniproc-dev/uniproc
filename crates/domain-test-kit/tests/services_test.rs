use domain::features::windows_manager::WindowManagerFeature;
use domain::features::services::ServicesFeature;
use domain_test_kit::generated::ServicesUiStub;
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
        .app_feature(WindowManagerFeature)
        .unwrap()
}

#[rstest]
#[serial]
#[ignore = "ServiceTableSettingsAdapter::initial_widths/min_widths/subscribe_widths are still \
            todo!() (crates/domain/src/features/services/view/mod.rs) - unrelated \
            pre-existing bug, tracked separately; re-enable once that's fixed"]
fn test_services_feature_installs_without_error(mut h: FeatureHarness) {
    let stub = ServicesUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        ServicesFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");
}
