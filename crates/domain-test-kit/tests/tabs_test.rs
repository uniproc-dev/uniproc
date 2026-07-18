use app_contracts::features::tabs::UiTabsPortMsg;
use domain::features::tabs::TabsFeature;
use domain_navigation::features::navigation::NavigationRegistryFeature;
use domain_test_kit::test_env::tabs::TabsPortStub;
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
        .app_feature(NavigationRegistryFeature)
        .unwrap()
}

#[rstest]
#[serial]
fn test_tabs_feature_sends_initial_tabs_on_install(mut h: FeatureHarness) {
    let stub = TabsPortStub::new();
    let port = stub.clone();

    h = h.window_feature(move || {
        let port = port.clone();
        TabsFeature::new(move |_: &DomainTestWindow| port.clone())
    });

    let ui_handle = h.0.as_ref().unwrap().ui().clone_strong();
    h.0.as_mut()
        .unwrap()
        .spawn_window(ui_handle)
        .expect("Failed to spawn window");

    assert!(
        stub.all()
            .iter()
            .any(|msg| matches!(msg, UiTabsPortMsg::SetTabs(_))),
        "installing the feature MUST push the bootstrap tabs to the UI"
    );
    assert!(
        stub.all()
            .iter()
            .any(|msg| matches!(msg, UiTabsPortMsg::SetAvailableContexts(_))),
        "installing the feature MUST push the bootstrap available contexts to the UI"
    );
}
