use amethystate::DefaultStore;
use app_contracts::features::sidebar::UiSidebarPortMsg;
use domain::features::sidebar::SidebarFeature;
use domain::features::sidebar::settings::SidebarSettings;
use domain_test_kit::test_env::sidebar::SidebarPortStub;
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
fn test_sidebar_feature_sends_initial_width_and_persists_ui_width_changes(mut h: FeatureHarness) {
    let stub = SidebarPortStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        SidebarFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    let ui_handle = h.0.as_ref().unwrap().ui().clone_strong();
    h.0.as_mut()
        .unwrap()
        .spawn_window(ui_handle)
        .expect("Failed to spawn window");

    assert_eq!(stub.all(), vec![UiSidebarPortMsg::SetSideBarWidth(260)]);

    stub.emit_side_bar_width_changed(500).stabilize(&mut h);

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
