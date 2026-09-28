use std::time::Duration;

use app_contracts::features::agents::{WindowsAction, WindowsReport};
use tokio::sync::Mutex;
use uniproc_windows_agent::agent::{Agent, Watch};

use super::windows_report::{self, Reports};

pub struct WindowsFeed {
    agent: Agent,
    interval: Box<dyn Fn() -> Duration + Send + Sync>,
    sampling: Mutex<Sampling>,
}

#[derive(Default)]
struct Sampling {
    watch: Option<(Duration, Watch)>,
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
        let Sampling { watch: slot, reports } = &mut *sampling;
        let watch = match &mut *slot {
            Some((held, watch)) if *held == interval => watch,
            stale => {
                *stale = None;
                let watching = self.agent.watch(windows_report::spec(interval)).await?;
                &mut stale.insert((interval, watching)).1
            }
        };
        match watch.next().await {
            Ok(update) => Ok(Some(reports.report(&update))),
            Err(error) => {
                *slot = None;
                Err(error)
            }
        }
    }

    pub async fn act(&self, action: WindowsAction) -> u32 {
        windows_report::code(self.agent.run(windows_report::command(action)).await)
    }
}
