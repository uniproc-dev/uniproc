use app_contracts::features::wsl::WslState;
use domain::features::wsl::WslFeature;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::wsl::{WslMsg, WslPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Wsl(WslPage);

#[page]
impl Page for Wsl {
    const CACHE_STATE_IN_MEMORY: bool = true;

    type Params = crate::routes::WslParams;
    type Installs = WslFeature;
    type Message = WslMsg;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn update(&mut self, message: WslMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, _) = cx.use_reducer::<WslState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: WslMsg| message);
        self.0.view(&state, &l10n, palette, forward)
    }
}
