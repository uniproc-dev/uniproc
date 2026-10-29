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
        let away = forward.clone();
        let bus = cx.bus();
        cx.use_effect_guard("uniproc::services::menu_closes_on_press_away", (), move || {
            bus.subscribe_fn(move |_: app_contracts::features::window::PressedAway| away.call(ServicesMsg::MenuDismiss))
        });
        let nav = cx.navigate::<ServicesRoute>();
        let settings = windows_reactor::Callback::new(move |()| { let _ = nav.to(ServicesRoute::Settings); });
        let operations = windows_reactor::component::<super::service_operations::ServiceOperations>("services-operations", ui::pages::services::OperationsProps {
            operations: state.operations.clone(), palette,
            collapse: l10n.services_operations_collapse(), expand: l10n.services_operations_expand(),
        }).into();
        self.0.view(&state, &dispatch, &l10n, palette, forward, settings, operations)
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

    #[derive(Default)]
    struct PendingServiceActions(std::sync::Mutex<Vec<futures::channel::oneshot::Sender<u32>>>);

    impl domain::features::agents::actions::ActsOnWindows for PendingServiceActions {
        fn act(&self, _: app_contracts::features::agents::WindowsAction) -> futures::future::BoxFuture<'static, u32> {
            let (send, recv) = futures::channel::oneshot::channel();
            self.0.lock().unwrap().push(send);
            Box::pin(async move { recv.await.unwrap_or(u32::MAX) })
        }
    }

    #[guinea::test(iterations = 4)]
    fn service_operations_follow_commands_and_the_panel_closes_when_empty(h: &mut Harness) {
        use domain::features::agents::actions::{self, WindowsTransport};
        use std::time::Duration;
        crate::test_support::install(h);
        h.app(|app| { actions::install(app); Ok(()) }).unwrap();
        h.install::<ServicesFeature>(&()).unwrap();
        let transport = Arc::new(PendingServiceActions::default());
        h.publish(WindowsTransport::Connected(transport.clone())).settle();
        h.publish(WindowsReportMessage::Report(Arc::new(WindowsReport {
            services: (0..4).map(|i| WindowsServiceStats { name: format!("sample-{i}").into(), display_name: format!("Sample {i}").into(), state: WindowsServiceState::Running, ..Default::default() }).collect(),
            ..Default::default()
        }))).settle();
        let mut page: Mounted<'_, Services> = Mounted::mount_at(h.child(), (), Route::Services(ServicesRoute::Overview)).unwrap();
        assert!(page.find(ServicesMark::Operations).is_none());
        let dispatch = h.dispatch::<ServicesState>();
        for i in 0..4 {
            dispatch.emit(Select(format!("sample-{i}")));
            h.settled();
            dispatch.emit(Command(ServiceActionKind::Stop));
            h.settled();
            assert_eq!(h.state::<ServicesState>().operations.len(), i + 1, "pending operations: {:?}", *h.state::<ServicesState>());
        }
        dispatch.emit(Command(ServiceActionKind::Restart));
        h.settled();
        assert_eq!(transport.0.lock().unwrap().len(), 4);
        assert_eq!(h.state::<ServicesState>().operations.len(), 4);
        h.advance(Duration::from_millis(40));
        h.settled();
        page.settle();
        assert!(page.find(ServicesMark::OperationsToggle).is_some());
        let scroll = page.find(ServicesMark::OperationsScroll).unwrap();
        assert_eq!(page.property(scroll, PropertyId::MaxHeight), Some(&PropertyValue::F64(132.0)));
        page.segment().act::<ServicesState>(Select("sample-0".into())).settle();
        assert_eq!(h.state::<ServicesState>().operations.len(), 4);
        page.click(ServicesMark::OperationsToggle).settle();
        page.settle();
        assert!(page.find(ServicesMark::OperationsScroll).is_none());
        assert!(page.find(ServicesMark::OperationsToggle).is_some());
        page.click(ServicesMark::OperationsToggle).settle();
        page.settle();
        assert!(page.find(ServicesMark::OperationsScroll).is_some());
        transport.0.lock().unwrap().remove(0).send(0).unwrap();
        h.settled(); page.settle();
        assert_eq!(h.state::<ServicesState>().operations.len(), 3);
        for send in transport.0.lock().unwrap().drain(..) { send.send(5).unwrap(); }
        h.settled(); page.settle();
        assert!(h.state::<ServicesState>().operations.is_empty());
        assert!(h.state::<ServicesState>().failure.is_some());
        h.advance(Duration::from_millis(250));
        h.settled();
        page.settle();
        if let Some(panel) = page.find(ServicesMark::Operations) {
            assert_eq!(page.property(panel, PropertyId::Opacity), Some(&PropertyValue::F64(0.0)));
        }
    }

    #[guinea::test(iterations = 4)]
    fn selected_service_keeps_its_slot_while_its_data_changes(h: &mut Harness) {
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let mut page: Mounted<'_, Services> = Mounted::mount_at(h.child(), (), Route::Services(ServicesRoute::Overview)).unwrap();
        let report = |beta: &str| WindowsReport { services: vec![
            WindowsServiceStats { name: "alpha".into(), display_name: "Alpha".into(), ..Default::default() },
            WindowsServiceStats { name: "beta".into(), display_name: beta.into(), ..Default::default() },
            WindowsServiceStats { name: "gamma".into(), display_name: "Gamma".into(), ..Default::default() },
        ], ..Default::default() };
        h.publish(WindowsReportMessage::Report(Arc::new(report("Beta")))).settle();
        page.segment().act::<ServicesState>(Select("beta".into())).settle();
        page.settle();
        assert_eq!(page.items().iter().position(selected_indicator), Some(1));
        h.publish(WindowsReportMessage::Report(Arc::new(report("Aardvark")))).settle();
        page.settle();
        let items = page.items();
        assert_eq!(items.iter().position(selected_indicator), Some(1));
        fn says(node: &Node, text: &str) -> bool {
            node.text.as_deref() == Some(text) || node.children.iter().any(|child| says(child, text))
        }
        assert!(says(&items[1], "Aardvark"));
        h.publish(app_contracts::features::window::PressedAway).settle();
        page.settle();
        assert!(says(&page.items()[0], "Aardvark"));
    }

    #[guinea::test(iterations = 4)]
    fn column_menu_persists_visibility_and_dismisses_on_press_away(h: &mut Harness) {
        use app_contracts::features::services::ServiceColumn;
        let store = crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let settings = store.try_settings::<ServicesSettings>().unwrap();
        let mut page: Mounted<'_, Services> = Mounted::mount_at(h.child(), (), Route::Services(ServicesRoute::Overview)).unwrap();
        h.publish(WindowsReportMessage::Report(Arc::new(WindowsReport::default()))).settle();
        page.settle();
        page.send(ServicesMsg::ColumnMenu);
        page.send(ServicesMsg::MenuAnchor { x: 40.0, y: 10.0 });
        page.settle();
        assert!(page.find(ServicesMark::Menu).is_some());
        let name = page.within(ServicesMark::Menu).find(ServiceColumn::Name).unwrap();
        assert_eq!(page.property(name, PropertyId::IsEnabled), Some(&PropertyValue::Bool(false)));
        page.within(ServicesMark::Menu).click(ServiceColumn::ImagePath).settle();
        page.settle();
        assert!(settings.columns().get(ServiceColumn::ImagePath.id()).unwrap().visible);
        assert!(page.find(ServicesMark::Menu).is_none());
        assert!(page.find(ServiceColumn::ImagePath).is_some());
        page.send(ServicesMsg::ColumnMenu);
        page.send(ServicesMsg::MenuAnchor { x: 80.0, y: 10.0 });
        page.settle();
        h.publish(app_contracts::features::window::PressedAway).settle();
        page.settle();
        assert!(page.find(ServicesMark::Menu).is_none());
    }

    #[guinea::test(iterations = 4)]
    fn sort_indicator_follows_the_sorted_column(h: &mut Harness) {
        use app_contracts::features::services::ServiceColumn;
        crate::test_support::install(h);
        h.install::<ServicesFeature>(&()).unwrap();
        let mut page: Mounted<'_, Services> = Mounted::mount_at(h.child(), (), Route::Services(ServicesRoute::Overview)).unwrap();
        h.publish(WindowsReportMessage::Report(Arc::new(WindowsReport::default()))).settle();
        page.settle();
        assert!(page.within(ServiceColumn::Name).find(ServicesMark::SortIndicator).is_some());
        page.click(ServiceColumn::Status).settle();
        page.settle();
        assert!(page.within(ServiceColumn::Status).find(ServicesMark::SortIndicator).is_some());
        assert!(page.within(ServiceColumn::Name).find(ServicesMark::SortIndicator).is_none());
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
