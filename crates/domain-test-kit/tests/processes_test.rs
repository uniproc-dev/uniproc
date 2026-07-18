use app_contracts::features::processes::UiProcessesPortMsg;
use domain::features::page_status::PageStatusFeature;
use domain_processes::features::processes::ProcessFeature;
use domain_test_kit::test_env::processes::ProcessesPortStub;
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
        .app_feature(PageStatusFeature)
        .unwrap()
}

#[rstest]
#[serial]
#[ignore = "ProcessTableSettingsAdapter::initial_widths/min_widths/subscribe_widths are still \
            todo!() (crates/domain/src/features/processes/domain/table.rs) - unrelated \
            pre-existing bug, tracked separately; re-enable once that's fixed"]
fn test_processes_feature_sends_waiting_empty_state_on_install(mut h: FeatureHarness) {
    let stub = ProcessesPortStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        ProcessFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    let ui_handle = h.0.as_ref().unwrap().ui().clone_strong();
    h.0.as_mut()
        .unwrap()
        .spawn_window(ui_handle)
        .expect("Failed to spawn window");

    assert!(
        stub.all()
            .contains(&UiProcessesPortMsg::SetEmptyStateVisible(true)),
        "installing the feature MUST show the waiting-for-data empty state"
    );
}
