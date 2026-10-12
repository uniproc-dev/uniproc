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

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, _) = cx.read::<WslState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: WslMsg| message);
        let (settings, _) = cx.read::<SettingsState>();
        self.0.view(&state, &l10n, palette, forward, settings.units.bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use app_contracts::features::agents::{LinuxMachineStats, RemoteScan, RemoteScanResult};
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

    fn sampled(h: &Harness, busy_ns: u64, at_ns: u64) {
        h.publish(RemoteScanResult::Scan(RemoteScan {
            schema_id: "wsl",
            processes: Vec::new(),
            machine: LinuxMachineStats {
                busy_ns,
                last_tsc: at_ns,
                cpu_count: 4,
                ..LinuxMachineStats::default()
            },
            environments: Vec::new(),
            docker_containers: Vec::new(),
        }))
        .settle();
    }

    #[guinea::test(iterations = 4)]
    fn the_machine_cpu_is_measured_over_the_time_between_the_agents_samples(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory()).unwrap();
        h.install::<WslFeature>(&WslDeps { scan: answering }).unwrap();
        let second = Duration::from_secs(1).as_nanos() as u64;

        sampled(h, 0, 10 * second);
        sampled(h, second, 11 * second);

        let percent = h.state::<WslState>().machine.as_ref().and_then(|machine| machine.cpu_percent);
        assert!(percent.is_some_and(|percent| (percent - 25.0).abs() < 0.01), "{percent:?}");
    }
}
