use std::rc::Rc;
use std::time::{Duration, Instant};

use amethystate::Field;
use app_contracts::features::agents::RemoteScanResult;
use app_contracts::features::wsl::{
    AgentPresence, DistroRow, LinuxMachineSummary, WslMsg, WslState,
};
use guinea::prelude::*;

use super::scanner::DistroScan;

#[derive(Clone, Copy, Debug)]
struct CpuSample {
    busy_ns: u64,
    at: Instant,
}

fn cpu_percent(previous: Option<CpuSample>, current: CpuSample, cpu_count: u32) -> Option<f32> {
    let previous = previous?;
    let cores = cpu_count.max(1) as f64;

    let elapsed_ns = current.at.duration_since(previous.at).as_nanos() as f64;
    if elapsed_ns <= 0.0 {
        return None;
    }

    let busy_ns = current.busy_ns.checked_sub(previous.busy_ns)? as f64;
    Some(((busy_ns / (elapsed_ns * cores)) * 100.0).clamp(0.0, 100.0) as f32)
}

#[derive(Debug)]
pub struct WslActor {
    ui_port: Push<WslState>,
    distros: Rc<[DistroRow]>,
    configured: Field<String>,
    machine: Option<LinuxMachineSummary>,
    previous_cpu: Option<CpuSample>,
    published: Option<(Rc<[DistroRow]>, Option<LinuxMachineSummary>)>,
    scan: fn(Duration) -> DistroScan,
    scanning: bool,
}

struct Scan;

#[expect(non_upper_case_globals)]
impl Scan {
    const Timeout: Duration = Duration::from_secs(10);
}

impl WslActor {
    pub fn new(ui_port: Push<WslState>, configured: Field<String>, scan: fn(Duration) -> DistroScan) -> Self {
        Self {
            ui_port,
            distros: Rc::from(Vec::new()),
            configured,
            machine: None,
            previous_cpu: None,
            published: None,
            scan,
            scanning: false,
        }
    }

    fn publish(&mut self) {
        let unchanged = self
            .published
            .as_ref()
            .is_some_and(|(distros, machine)| *distros == self.distros && *machine == self.machine);
        if unchanged {
            return;
        }
        self.published = Some((self.distros.clone(), self.machine.clone()));
        self.ui_port.send(WslMsg::Set {
            distros: self.distros.clone(),
            machine: self.machine.clone(),
        });
    }

    fn agent_distro(&self) -> Option<String> {
        let configured = self.configured.get();
        if !configured.is_empty() {
            return Some(configured);
        }
        self.distros.iter().find(|row| row.is_default).map(|row| row.name.clone())
    }

    fn apply_presence(&mut self) {
        let answering = self.machine.is_some();
        let Some(agent_distro) = self.agent_distro() else {
            return;
        };

        let mut rows = self.distros.to_vec();
        for row in &mut rows {
            if row.name == agent_distro {
                row.agent = if answering {
                    AgentPresence::Answering
                } else {
                    AgentPresence::Silent
                };
                if self.machine.is_some() {
                    row.metrics = self.machine.clone();
                }
            }
        }
        self.distros = Rc::from(rows);
    }

    fn carry_known_metrics(&self, rows: &mut [DistroRow]) {
        for row in rows {
            if row.metrics.is_none()
                && let Some(known) = self.distros.iter().find(|previous| previous.name == row.name)
            {
                row.metrics = known.metrics.clone();
            }
        }
    }

    fn forget_metrics(&mut self) {
        let Some(agent_distro) = self.agent_distro() else {
            return;
        };

        let mut rows = self.distros.to_vec();
        for row in &mut rows {
            if row.name == agent_distro {
                row.metrics = None;
            }
        }
        self.distros = Rc::from(rows);
    }
}

#[derive(Clone, Debug, serde::Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RefreshDistros;

enum ScanResult {
    Distros(Vec<DistroRow>),
    Failed,
}

actor! {
    WslActor {
        handlers { RefreshDistros, ScanResult, RemoteScanResult }
    }
}

#[handler]
fn handle_refresh(this: &mut WslActor, _: RefreshDistros, cx: Cx) {
    if this.scanning {
        return;
    }
    this.scanning = true;
    let scan = (this.scan)(Scan::Timeout);
    cx.spawn_bg(async move {
        match scan.await {
            Ok(distros) => ScanResult::Distros(distros),
            Err(err) => {
                tracing::warn!(%err, "wsl distribution scan failed");
                ScanResult::Failed
            }
        }
    });
}

#[handler]
fn on_scan_result(this: &mut WslActor, msg: ScanResult) {
    this.scanning = false;
    let ScanResult::Distros(mut distros) = msg else {
        return;
    };
    this.carry_known_metrics(&mut distros);
    this.distros = Rc::from(distros);
    this.apply_presence();
    this.publish();
}

#[handler]
fn on_remote_scan(this: &mut WslActor, msg: RemoteScanResult) {
    match msg {
        RemoteScanResult::Scan(scan) => {
            let sample = CpuSample {
                busy_ns: scan.machine.busy_ns,
                at: Instant::now(),
            };
            let known_percent = this.machine.as_ref().and_then(|m| m.cpu_percent);
            let percent =
                cpu_percent(this.previous_cpu, sample, scan.machine.cpu_count).or(known_percent);
            this.previous_cpu = Some(sample);

            let m = &scan.machine;
            this.machine = Some(LinuxMachineSummary {
                cpu_percent: percent,
                memory_used_bytes: m.used_kb * 1024,
                memory_total_bytes: m.total_kb * 1024,
                disk_bytes: m.disk_read_bytes + m.disk_write_bytes,
                net_bytes: m.tcp_rx_remote_bytes
                    + m.tcp_tx_remote_bytes
                    + m.udp_rx_remote_bytes
                    + m.udp_tx_remote_bytes,
                process_count: scan.processes.len(),
                container_count: scan.docker_containers.len(),
            });
        }
        RemoteScanResult::Unavailable(_) => {
            this.machine = None;
            this.previous_cpu = None;
            this.forget_metrics();
        }
    }
    this.apply_presence();
    this.publish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample(busy_ns: u64, at: Instant) -> CpuSample {
        CpuSample { busy_ns, at }
    }

    #[test]
    fn the_first_report_has_nothing_to_compare_against() {
        let now = Instant::now();
        assert_eq!(cpu_percent(None, sample(1_000, now), 4), None);
    }

    #[test]
    fn busy_time_is_divided_by_cores() {
        let start = Instant::now();
        let later = start + Duration::from_secs(1);

        let all_four_cores = cpu_percent(Some(sample(0, start)), sample(4_000_000_000, later), 4);
        assert!((all_four_cores.unwrap() - 100.0).abs() < 0.5);

        let one_of_four = cpu_percent(Some(sample(0, start)), sample(1_000_000_000, later), 4);
        assert!((one_of_four.unwrap() - 25.0).abs() < 0.5);
    }

    #[test]
    fn a_restarted_agent_counting_from_zero_yields_nothing() {
        let start = Instant::now();
        let later = start + Duration::from_secs(1);

        assert_eq!(
            cpu_percent(Some(sample(9_000_000_000, start)), sample(10, later), 4),
            None
        );
    }

    #[test]
    fn a_missing_core_count_is_treated_as_one() {
        let start = Instant::now();
        let later = start + Duration::from_secs(1);

        let percent = cpu_percent(Some(sample(0, start)), sample(1_000_000_000, later), 0);
        assert!((percent.unwrap() - 100.0).abs() < 0.5);
    }

    fn distro(name: &str, running: bool) -> DistroRow {
        DistroRow {
            name: name.to_string(),
            running,
            is_default: false,
            agent: AgentPresence::NotChecked,
            metrics: None,
        }
    }

    fn detached_port() -> Push<WslState> {
        let scope = Rc::new(guinea::core::scope::Scope::new());
        let token = guinea::core::actor::UiThreadToken::dangerously_create_token_unchecked();
        let registry = Rc::new(guinea::core::actor::registry::DebugRegistry::new());
        let bus = Rc::new(guinea::core::actor::event_bus::EventBus::new());
        guinea::core::feature::Claim::<WslState>::new(&scope, &bus, &token, &registry)
            .plain()
            .port()
    }

    fn configured_for(distro: &str, distros: Vec<DistroRow>) -> WslActor {
        let configured = Field::new_volatile(["wsl-test", "distro"], distro.to_string());
        let mut actor = WslActor::new(detached_port(), configured, |_| Box::pin(std::future::pending()));
        actor.distros = Rc::from(distros);
        actor
    }

    fn actor_with(distros: Vec<DistroRow>) -> WslActor {
        configured_for("Ubuntu", distros)
    }

    #[test]
    fn with_no_distribution_chosen_the_agent_is_in_the_wsl_default() {
        let mut actor = configured_for(
            "",
            vec![distro("Ubuntu", true), DistroRow { is_default: true, ..distro("Ubuntu-24.04", true) }],
        );
        actor.apply_presence();

        assert_eq!(actor.distros[0].agent, AgentPresence::NotChecked);
        assert_eq!(actor.distros[1].agent, AgentPresence::Silent);
    }

    #[test]
    fn only_the_configured_distribution_is_judged() {
        let mut actor = actor_with(vec![distro("Ubuntu", true), distro("Debian", true)]);
        actor.apply_presence();

        assert_eq!(actor.distros[0].agent, AgentPresence::Silent);
        assert_eq!(
            actor.distros[1].agent,
            AgentPresence::NotChecked,
            "a distribution nothing was attempted against is not agentless"
        );
    }

    #[test]
    fn a_report_makes_the_configured_distribution_answering() {
        let mut actor = actor_with(vec![distro("Ubuntu", true)]);
        actor.machine = Some(LinuxMachineSummary::default());
        actor.apply_presence();

        assert_eq!(actor.distros[0].agent, AgentPresence::Answering);
    }

    #[test]
    fn losing_the_agent_takes_the_figures_with_it() {
        let mut actor = actor_with(vec![distro("Ubuntu", true)]);
        actor.machine = Some(LinuxMachineSummary::default());
        actor.apply_presence();

        actor.machine = None;
        actor.forget_metrics();
        actor.apply_presence();

        assert_eq!(actor.distros[0].agent, AgentPresence::Silent);
        assert_eq!(actor.distros[0].metrics, None);
    }

    #[test]
    fn a_report_that_has_not_arrived_yet_leaves_the_figures_alone() {
        let mut actor = actor_with(vec![distro("Ubuntu", true)]);
        actor.machine = Some(LinuxMachineSummary {
            memory_used_bytes: 512,
            ..LinuxMachineSummary::default()
        });
        actor.apply_presence();

        actor.machine = None;
        actor.apply_presence();

        assert_eq!(
            actor.distros[0].metrics.as_ref().map(|m| m.memory_used_bytes),
            Some(512),
            "a fresh actor with no report yet must not blank what the page already showed"
        );
    }

    #[test]
    fn a_rescan_carries_the_known_figures_onto_the_new_rows() {
        let actor = actor_with(vec![DistroRow {
            metrics: Some(LinuxMachineSummary {
                memory_used_bytes: 512,
                ..LinuxMachineSummary::default()
            }),
            ..distro("Ubuntu", true)
        }]);

        let mut scanned = vec![distro("Ubuntu", true)];
        actor.carry_known_metrics(&mut scanned);

        assert_eq!(
            scanned[0].metrics.as_ref().map(|m| m.memory_used_bytes),
            Some(512)
        );
    }
}
