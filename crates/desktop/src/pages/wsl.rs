use app_contracts::features::settings::SettingsState;
use app_contracts::features::wsl::WslState;
use domain::features::wsl::{WslDeps, WslFeature};
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
        ctx.install(&ctx.require_or_default::<WslDeps>())
    }

    fn update(&mut self, message: WslMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, _) = cx.read::<WslState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: WslMsg| message);
        let (settings, _) = cx.read::<SettingsState, _>();
        self.0.view(&state, &l10n, palette, forward, settings.units.bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use domain::features::wsl::DistroScan;
    use guinea::app::Harness;
    use guinea_plugin_store::StorePlugin;

    use super::*;

    thread_local! {
        static SCANS: Cell<u32> = const { Cell::new(0) };
    }

    fn hanging(_: Duration) -> DistroScan {
        SCANS.set(SCANS.get() + 1);
        Box::pin(std::future::pending())
    }

    fn answering(_: Duration) -> DistroScan {
        SCANS.set(SCANS.get() + 1);
        Box::pin(async { Ok(Vec::new()) })
    }

    fn scans_in_ten_seconds(h: &mut Harness, scan: fn(Duration) -> DistroScan) -> u32 {
        SCANS.set(0);
        h.plugin(StorePlugin::in_memory()).unwrap();
        h.install::<WslFeature>(&WslDeps { scan }).unwrap();
        h.advance(Duration::from_secs(10));
        SCANS.get()
    }

    #[guinea::test(iterations = 4)]
    fn a_wsl_that_does_not_answer_is_not_asked_again(h: &mut Harness) {
        assert_eq!(scans_in_ten_seconds(h, hanging), 1);
    }

    #[guinea::test(iterations = 4)]
    fn a_wsl_that_answers_is_asked_every_tick(h: &mut Harness) {
        assert!(scans_in_ten_seconds(h, answering) > 1);
    }
}
