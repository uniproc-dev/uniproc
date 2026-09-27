use std::time::Duration;

use app_contracts::features::agents::{WindowsAction, WindowsReport};
use tokio::sync::Mutex;
use uniproc_windows_agent::agent::{Agent, Sampler};

use super::windows_report::{self, Reports};

pub struct WindowsFeed {
    agent: Agent,
    interval: Box<dyn Fn() -> Duration + Send + Sync>,
    sampling: Mutex<Sampling>,
}

#[derive(Default)]
struct Sampling {
    sampler: Option<(Duration, Sampler)>,
    reports: Reports,
}

impl WindowsFeed {
    pub fn new(agent: Agent, interval: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            agent,
            interval: Box::new(interval),
            sampling: Mutex::default(),
        }
    }

    pub fn agent(&self) -> &Agent {
        &self.agent
    }

    pub async fn report(&self) -> anyhow::Result<Option<WindowsReport>> {
        let interval = (self.interval)();
        let mut sampling = self.sampling.lock().await;
        let Sampling { sampler, reports } = &mut *sampling;
        let sampler = match sampler {
            Some((held, sampler)) if *held == interval => sampler,
            slot => {
                *slot = None;
                let subscribed = self.agent.subscribe(windows_report::spec(interval)).await?;
                &mut slot.insert((interval, subscribed)).1
            }
        };
        let sample = sampler.next().await?;
        let Some(snapshot) = self.agent.snapshot().await? else {
            return Ok(None);
        };
        Ok(Some(reports.report(&snapshot, &sample)))
    }

    pub async fn act(&self, action: WindowsAction) -> u32 {
        windows_report::code(self.agent.run(windows_report::command(action)).await)
    }
}
