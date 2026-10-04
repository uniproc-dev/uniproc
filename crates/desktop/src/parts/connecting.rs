use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::AgentConnectionState;
use domain::features::agent_link::{AgentLinkDeps, AgentLinkFeature};
use domain::features::agents::providers::windows::SERVICE_DISPLAY_NAME;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx};
use ui::theme::{scheme_context, Palette};
use ui::widgets::nothing;
use windows_reactor::{Callback, View};

#[derive(Default)]
pub struct Connecting;

fn trouble(windows: AgentConnectionState) -> Option<ui::ServiceTrouble<'static>> {
    match windows {
        AgentConnectionState::GaveUp => Some(ui::ServiceTrouble::Unreachable(SERVICE_DISPLAY_NAME)),
        AgentConnectionState::Outdated => Some(ui::ServiceTrouble::Outdated(SERVICE_DISPLAY_NAME)),
        _ => None,
    }
}

#[page]
impl Page for Connecting {
    type Installs = AgentLinkFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&ctx.require_or_default::<AgentLinkDeps>())
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (link, dispatch) = cx.read::<AgentLinkState>();
        if !link.awaiting_first_connection() {
            return nothing();
        }
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        ui::splash_view(ui::SplashProps {
            l10n: &l10n,
            palette,
            in_process_offered: link.in_process_offered,
            in_process: link.in_process,
            service_trouble: trouble(link.windows),
            on_start_in_process: Callback::new(move |()| dispatch.emit(StartInProcess)),
        })
    }
}
