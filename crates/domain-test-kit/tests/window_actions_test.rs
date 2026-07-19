use app_contracts::features::window_actions::{ResizeEdge, UiWindowActionsPortMsg};
use domain::features::window_actions::WindowActionsFeature;
use domain_test_kit::generated::WindowActionsUiStub;
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
fn test_window_actions_feature_forwards_ui_events_to_port(mut h: FeatureHarness) {
    let stub = WindowActionsUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        WindowActionsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");

    stub.emit_on_drag().stabilize(&mut h);
    assert_eq!(
        stub.ui_window_actions_port_sent().stabilize(&mut h),
        vec![UiWindowActionsPortMsg::Drag]
    );

    stub.emit_on_close().stabilize(&mut h);
    assert_eq!(
        stub.ui_window_actions_port_sent().stabilize(&mut h),
        vec![UiWindowActionsPortMsg::Drag, UiWindowActionsPortMsg::Close]
    );

    stub.emit_on_start_resize(ResizeEdge::East).stabilize(&mut h);
    assert_eq!(
        stub.ui_window_actions_port_sent().stabilize(&mut h).last(),
        Some(&UiWindowActionsPortMsg::Resize(ResizeEdge::East))
    );
}
