use app_contracts::features::environments::UiEnvironmentsPortMsg;
use domain_environments::features::environments::EnvironmentsFeature;
use domain_test_kit::generated::EnvironmentsUiStub;
use domain_test_kit::utils::{DomainTestWindow, FeatureHarness, temp_settings_path};
use forsl::settings::SettingsFeature;
use forsl_core::test_kit::Stabilizer;
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
#[ignore = "check_wsl_availability_async shells out to the real wsl.exe and currently \
            crashes with STATUS_STACK_BUFFER_OVERRUN - unrelated pre-existing bug, tracked \
            separately; re-enable once that's fixed"]
fn test_environments_feature_starts_wsl_status_check_on_install(mut h: FeatureHarness) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let stub = EnvironmentsUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        EnvironmentsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");

    h.stabilize();

    assert!(
        stub.ui_environments_port_sent()
            .stabilize(&mut h)
            .contains(&UiEnvironmentsPortMsg::SetWslIsLoading(true)),
        "installing the feature MUST kick off a WSL status check"
    );
}
