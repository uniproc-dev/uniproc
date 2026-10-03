use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agents::{WindowsAction, WindowsMachineSample, WindowsProcessEvents, WindowsReport};
use app_contracts::features::settings::UpdateInterval;
use tokio::sync::Mutex;
use uniproc_windows_agent::agent::{Agent, Watch};
use uniproc_windows_agent::api::{MetricSpec, ProcessEventBatch, Update};
use uniproc_windows_agent::local::ProcessEventsWatch;
use uniproc_windows_agent::remote::RemoteProcessEvents;

use super::process_events;
use super::windows_report::{self, Reports};

pub struct WindowsFeed {
    agent: Agent,
    interval: Box<dyn Fn() -> Duration + Send + Sync>,
    sampling: Mutex<Sampling>,
    machine: Mutex<Sampling>,
    events: Mutex<Option<EventsWatch>>,
}

#[derive(Default)]
struct Sampling {
    watch: Option<(Duration, Watch)>,
    reports: Reports,
}

impl Sampling {
    async fn next(
        &mut self,
        agent: &Agent,
        interval: Duration,
        spec: fn(Duration) -> MetricSpec,
    ) -> anyhow::Result<(&mut Reports, Update)> {
        let Sampling { watch: slot, reports } = self;
        let watch = match &mut *slot {
            Some((held, watch)) if *held == interval => watch,
            stale => {
                *stale = None;
                let watching = agent.watch(spec(interval)).await?;
                &mut stale.insert((interval, watching)).1
            }
        };
        match watch.next().await {
            Ok(update) => Ok((reports, update)),
            Err(error) => {
                *slot = None;
                Err(error)
            }
        }
    }
}

impl WindowsFeed {
    pub fn new(agent: Agent, interval: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            agent,
            interval: Box::new(interval),
            sampling: Mutex::default(),
            machine: Mutex::default(),
            events: Mutex::default(),
        }
    }

    pub fn agent(&self) -> &Agent {
        &self.agent
    }

    pub async fn report(&self) -> anyhow::Result<Option<WindowsReport>> {
        let interval = (self.interval)();
        let mut sampling = self.sampling.lock().await;
        let (reports, update) = sampling.next(&self.agent, interval, windows_report::spec).await?;
        Ok(Some(reports.report(&update)))
    }

    pub async fn machine(&self) -> anyhow::Result<WindowsMachineSample> {
        let interval = (self.interval)().min(UpdateInterval::Machine);
        let mut sampling = self.machine.lock().await;
        let (reports, update) = sampling.next(&self.agent, interval, windows_report::machine_spec).await?;
        Ok(WindowsMachineSample {
            machine: Arc::new(reports.machine_sample(&update.sample)),
            clock_100ns: update.sample.sampled_at,
        })
    }

    pub async fn act(&self, action: WindowsAction) -> u32 {
        windows_report::code(self.agent.run(windows_report::command(action)).await)
    }

    pub async fn process_events(&self) -> anyhow::Result<WindowsProcessEvents> {
        let mut slot = self.events.lock().await;
        let watch = match &mut *slot {
            Some(watch) => watch,
            None => {
                let opened = match &self.agent {
                    Agent::Local(local) => EventsWatch::Local(local.watch_process_events()),
                    Agent::Remote(remote) if remote.can_watch_process_events() => {
                        EventsWatch::Remote(remote.watch_process_events().await?)
                    }
                    Agent::Remote(_) => return std::future::pending().await,
                };
                slot.insert(opened)
            }
        };
        match watch.next().await {
            Ok(told) => Ok(process_events::batch(told)),
            Err(error) => {
                *slot = None;
                Err(error)
            }
        }
    }
}

enum EventsWatch {
    Remote(RemoteProcessEvents),
    Local(ProcessEventsWatch),
}

impl EventsWatch {
    async fn next(&mut self) -> anyhow::Result<ProcessEventBatch> {
        match self {
            Self::Remote(watch) => watch.next().await,
            Self::Local(watch) => watch
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("monitoring in process stopped")),
        }
    }
}
