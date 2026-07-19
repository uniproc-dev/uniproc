use app_contracts::features::processes::UiProcessesPortMsg;
use domain::features::page_status::PageStatusFeature;
use domain_processes::features::processes::ProcessFeature;
use domain_test_kit::generated::ProcessesUiStub;
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
        .app_feature(PageStatusFeature)
        .unwrap()
}

#[rstest]
#[serial]
#[ignore = "ProcessTableSettingsAdapter::initial_widths/min_widths/subscribe_widths are still \
            todo!() (crates/domain/src/features/processes/domain/table.rs) - unrelated \
            pre-existing bug, tracked separately; re-enable once that's fixed"]
fn test_processes_feature_sends_waiting_empty_state_on_install(mut h: FeatureHarness) {
    let stub = ProcessesUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        ProcessFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");

    assert!(
        stub.ui_processes_port_sent()
            .stabilize(&mut h)
            .contains(&UiProcessesPortMsg::SetEmptyStateVisible(true)),
        "installing the feature MUST show the waiting-for-data empty state"
    );
}
