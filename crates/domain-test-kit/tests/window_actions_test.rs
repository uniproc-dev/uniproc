use app_contracts::features::window_actions::{ResizeEdge, UiWindowActionsPortMsg};
use domain::features::window_actions::WindowActionsFeature;
use domain_test_kit::test_env::window_actions::WindowActionsPortStub;
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
fn test_window_actions_feature_forwards_ui_events_to_port(mut h: FeatureHarness) {
    let stub = WindowActionsPortStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        WindowActionsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    let ui_handle = h.0.as_ref().unwrap().ui().clone_strong();
    h.0.as_mut()
        .unwrap()
        .spawn_window(ui_handle)
        .expect("Failed to spawn window");

    stub.emit_drag().stabilize(&mut h);
    assert_eq!(stub.all(), vec![UiWindowActionsPortMsg::Drag]);

    stub.emit_close().stabilize(&mut h);
    assert_eq!(
        stub.all(),
        vec![UiWindowActionsPortMsg::Drag, UiWindowActionsPortMsg::Close]
    );

    stub.emit_start_resize(ResizeEdge::East).stabilize(&mut h);
    assert_eq!(
        stub.all().last(),
        Some(&UiWindowActionsPortMsg::Resize(ResizeEdge::East))
    );
}
