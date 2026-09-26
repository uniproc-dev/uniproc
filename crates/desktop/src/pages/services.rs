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

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<ServicesState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ServicesMsg| message);
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}
