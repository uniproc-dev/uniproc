use app_contracts::features::services::ServicesState;
use domain::features::services::ServicesFeature;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::services::{ServicesMsg, ServicesPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Services(ServicesPage);

#[page]
impl Page for Services {
    const CACHE_STATE_IN_MEMORY: bool = true;

    type Params = crate::routes::ServicesParams;
    type Installs = ServicesFeature;
    type Message = ServicesMsg;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn update(&mut self, message: ServicesMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, dispatch) = cx.read::<ServicesState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ServicesMsg| message);
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::agents::{
        ActionOutcome, WindowsActionRequest, WindowsReport, WindowsReportMessage, WindowsServiceState,
        WindowsServiceStats,
    };
    use guinea::app::Harness;
    use guinea::prelude::GlobalEventBus;
    use guinea::winui::harness::{Mounted, PropertyId, PropertyValue};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::services::ServicesMark;
    use ui::widgets::action_failure::ActionFailureMark;

    use super::*;
    use crate::routes::Route;

    fn tip(page: &Mounted<'_, Services>, property: PropertyId) -> Option<PropertyValue> {
        let tip = page.find(ActionFailureMark::Tip).expect("the failure tip is on the page");
        page.property(tip, property).cloned()
    }

    #[guinea::test(iterations = 8)]
    fn a_service_that_will_not_stop_says_why(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap();
        let h = &*h;
        let mut page =
            Mounted::mount_at(h.child(), crate::routes::ServicesParams::default(), Route::Services {}).unwrap();
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
        let _service = GlobalEventBus::answer_fn(|_: WindowsActionRequest| ActionOutcome::Busy);

        page.settle();
        page.item(0).click_here();
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
}
