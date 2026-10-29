use app_contracts::features::services::ServicesState;
use guinea::feature::Install;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::services::{ServicesMsg, ServicesPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;
use guinea_plugin_store::StoreAccess;
use ui::pages::services::ServicesSettingsMaps;
use domain::features::services::settings::ServicesSettings;
use crate::routes::ServicesRoute;

pub(super) fn open_settings(store: &amethystate::Store) -> anyhow::Result<ServicesSettingsMaps> {
    let settings = store.try_settings::<ServicesSettings>()?;
    Ok(ServicesSettingsMaps { columns: settings.columns().clone(), order: settings.order().clone() })
}

#[derive(Default)]
pub struct Services(ServicesPage);

#[page]
impl Page for Services {
    type Message = ServicesMsg;

    fn install(_ctx: &Install, store: amethystate::Store) -> anyhow::Result<Self> {
        Ok(Self(ServicesPage::new(open_settings(&store)?)))
    }

    fn update(&mut self, message: ServicesMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, dispatch) = cx.read::<ServicesState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ServicesMsg| message);
        let nav = cx.navigate::<ServicesRoute>();
        let settings = windows_reactor::Callback::new(move |()| { let _ = nav.to(ServicesRoute::Settings); });
        self.0.view(&state, &dispatch, &l10n, palette, forward, settings)
    }
}

#[cfg(test)]
mod tests {
    use domain::features::services::ServicesFeature;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use app_contracts::features::agents::{
        ActionOutcome, WindowsActionRequest, WindowsReport, WindowsReportMessage, WindowsServiceState,
        WindowsServiceStats,
    };
    use app_contracts::features::services::{Command, Select, ServiceActionKind};
    use domain::features::agents::providers::windows::{SERVICE_DISPLAY_NAME, SERVICE_NAME};
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node, PropertyId, PropertyValue};
    use ui::pages::services::ServicesMark;
    use ui::widgets::action_failure::ActionFailureMark;

    use super::*;
    use crate::routes::Route;

    fn selected_indicator(node: &Node) -> bool {
        node.id.as_deref() == Some(guinea::Mark::name(&ServicesMark::SelectedIndicator))
            || node.children.iter().any(selected_indicator)
    }

    #[guinea::test(iterations = 4)]
    fn selected_service_has_an_indicator_and_is_cleared_outside_the_table(h: &mut Harness) {
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let mut page: Mounted<'_, Services> = Mounted::mount_at(h.child(), (),
            Route::Services(ServicesRoute::Overview)).unwrap();
        h.publish(WindowsReportMessage::Report(Arc::new(WindowsReport {
            services: vec![WindowsServiceStats { name: "sample".into(), display_name: "Sample".into(),
                state: WindowsServiceState::Running, ..Default::default() }], ..Default::default()
        }))).settle();
        page.segment().act::<ServicesState>(Select("sample".into())).settle();
        page.settle();
        assert!(page.items().iter().any(selected_indicator));
        h.publish(app_contracts::features::window::PressedAway).settle();
        page.settle();
        assert!(!page.items().iter().any(selected_indicator));
        page.segment().act::<ServicesState>(Select("sample".into())).settle();
        page.settle();
        page.click(ui::widgets::table::TableMark::EmptyArea).settle();
        page.settle();
        assert!(!page.items().iter().any(selected_indicator));
    }

    #[guinea::test(iterations = 1, exclusive = "services-toolbar")]
    fn toolbar_launches_a_task_and_the_services_console(h: &mut Harness) {
        static TASKS: AtomicU32 = AtomicU32::new(0);
        static CONSOLES: AtomicU32 = AtomicU32::new(0);
        TASKS.store(0, Ordering::SeqCst);
        CONSOLES.store(0, Ordering::SeqCst);
        let mut deps = crate::platform::PlatformDeps::default();
        deps.processes.shell = |request| {
            assert_eq!(request, domain::features::processes::shell::ShellRequest::RunNewTask);
            TASKS.fetch_add(1, Ordering::SeqCst);
        };
        deps.system.launch = |launch| {
            assert_eq!(launch, domain::features::system::tools::Launch {
                file: "services.msc".into(),
                args: String::new(),
            });
            CONSOLES.fetch_add(1, Ordering::SeqCst);
        };
        crate::test_support::install_with(h, deps);
        h.install::<ServicesFeature>(&()).unwrap();
        let mut page: Mounted<'_, Services> =
            Mounted::mount_at(h.child(), (), Route::Services(crate::routes::ServicesRoute::Overview)).unwrap();
        page.settle();
        page.click(ServicesMark::RunNewTask).settle();
        page.click(ServicesMark::OpenServices).settle();
        assert_eq!(TASKS.load(Ordering::SeqCst), 1);
        assert_eq!(CONSOLES.load(Ordering::SeqCst), 1);
    }

    #[guinea::test(iterations = 1)]
    fn services_use_the_native_scroll_viewer(h: &mut Harness) {
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let mut page: Mounted<'_, Services> =
            Mounted::mount_at(h.child(), (), Route::Services(crate::routes::ServicesRoute::Overview)).unwrap();
        h.publish(WindowsReportMessage::Report(Arc::new(WindowsReport {
            services: vec![WindowsServiceStats {
                name: "sample-service".into(),
                display_name: "Sample service".into(),
                state: WindowsServiceState::Running,
                ..Default::default()
            }],
            ..Default::default()
        }))).settle();
        page.settle();

        assert!(page.find(ui::widgets::table::TableMark::Scroll).is_some());
    }

    fn tip(page: &Mounted<'_, Services>, property: PropertyId) -> Option<PropertyValue> {
        let tip = page.find(ActionFailureMark::Tip).expect("the failure tip is on the page");
        page.property(tip, property).cloned()
    }

    #[guinea::test(iterations = 8)]
    fn a_service_that_will_not_stop_says_why(h: &mut Harness) {
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let h = &*h;
        let mut page =
            Mounted::mount_at(h.child(), (), Route::Services(crate::routes::ServicesRoute::Overview)).unwrap();
        let report = WindowsReport {
            services: vec![WindowsServiceStats {
                name: "Audiosrv".into(),
                display_name: "Windows Audio".into(),
                state: WindowsServiceState::Running,
                ..Default::default()
            }],
            ..Default::default()
        };
        h.publish(WindowsReportMessage::Report(Arc::new(report))).settle();
        let _service = h.bus().answer_fn(|_: WindowsActionRequest| ActionOutcome::Busy);

        page.settle();
        page.segment().act::<ServicesState>(Select("Audiosrv".into())).settle();
        page.settle();
        assert_eq!(tip(&page, PropertyId::IsOpen), Some(PropertyValue::Bool(false)));
        page.click(ServicesMark::Stop).settle();
        page.settle();
        assert_eq!(tip(&page, PropertyId::IsOpen), Some(PropertyValue::Bool(true)));
        let Some(PropertyValue::String(title)) = tip(&page, PropertyId::Title) else {
            panic!("the tip has a title");
        };
        assert_eq!(title.replace(['\u{2068}', '\u{2069}'], ""), "Couldn’t stop Windows Audio");
        assert_eq!(
            tip(&page, PropertyId::Subtitle),
            Some(PropertyValue::String("Another action on it hasn’t finished yet.".into()))
        );
    }

    #[guinea::test(iterations = 8)]
    fn uniproc_does_not_stop_its_own_service(h: &mut Harness) {
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let h = &*h;
        let mut page: Mounted<'_, Services> =
            Mounted::mount_at(h.child(), (), Route::Services(crate::routes::ServicesRoute::Overview)).unwrap();
        let report = WindowsReport {
            services: vec![WindowsServiceStats {
                name: SERVICE_NAME.into(),
                display_name: SERVICE_DISPLAY_NAME.into(),
                state: WindowsServiceState::Running,
                ..Default::default()
            }],
            ..Default::default()
        };
        h.publish(WindowsReportMessage::Report(Arc::new(report))).settle();
        let asked = Arc::new(AtomicU32::new(0));
        let counted = asked.clone();
        let _service = h.bus().answer_fn(move |_: WindowsActionRequest| {
            counted.fetch_add(1, Ordering::SeqCst);
            ActionOutcome::Done
        });

        page.settle();
        page.segment().act::<ServicesState>(Select(SERVICE_NAME.into())).settle();
        for kind in [ServiceActionKind::Stop, ServiceActionKind::Pause, ServiceActionKind::Restart] {
            page.segment().act::<ServicesState>(Command(kind)).settle();
        }
        page.settle();

        assert_eq!(asked.load(Ordering::SeqCst), 0);
    }
}
