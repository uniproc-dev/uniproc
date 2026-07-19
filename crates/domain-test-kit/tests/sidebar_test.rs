use amethystate::DefaultStore;
use app_contracts::features::sidebar::UiSidebarPortMsg;
use domain::features::sidebar::SidebarFeature;
use domain::features::sidebar::settings::SidebarSettings;
use domain_test_kit::generated::SidebarUiStub;
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
fn test_sidebar_feature_sends_initial_width_and_persists_ui_width_changes(mut h: FeatureHarness) {
    let stub = SidebarUiStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        SidebarFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    h.spawn_window().expect("Failed to spawn window");

    assert_eq!(
        stub.ui_sidebar_port_sent().stabilize(&mut h),
        vec![UiSidebarPortMsg::SetSideBarWidth(260)]
    );

    stub.emit_on_side_bar_width_changed(500).stabilize(&mut h);

    let store = h
        .shared()
        .get::<DefaultStore>()
        .expect("DefaultStore must be installed");
    let settings = SidebarSettings::new_with(store.as_ref()).unwrap();
    assert_eq!(
        settings.width().get(),
        500,
        "width change from the UI MUST persist to settings"
    );
}
