use std::sync::Arc;

use app_contracts::features::agents::{WindowsAction, WindowsReport};
use futures::future::BoxFuture;

pub enum InProcessStartError {
    NotElevated,
    Failed(String),
}

pub trait InProcessAgent: Send + Sync + 'static {
    fn report(self: Arc<Self>) -> BoxFuture<'static, anyhow::Result<Option<WindowsReport>>>;
    fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32>;
}

pub type InProcessStart =
    fn() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>>;

pub use local::start_local;

mod local {
    use std::sync::Arc;

    use app_contracts::features::agents::{WindowsAction, WindowsReport};
    use app_contracts::features::settings::UpdateInterval;
    use futures::future::BoxFuture;
    use uniproc_windows_agent::agent::Agent;
    use uniproc_windows_agent::local::{Local, StartError};

    use super::{InProcessAgent, InProcessStartError};
    use crate::features::agents::windows_feed::WindowsFeed;
    use crate::features::settings::settings::GeneralSettings;

    struct LocalAgent {
        feed: WindowsFeed,
    }

    pub fn start_local() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>> {
        Box::pin(async {
            let interval = GeneralSettings::new()
                .map_err(|error| InProcessStartError::Failed(error.to_string()))?
                .update_interval_ms();
            match tokio::task::spawn_blocking(Local::start).await {
                Ok(Ok(local)) => Ok(Arc::new(LocalAgent {
                    feed: WindowsFeed::new(Agent::Local(Arc::new(local)), move || {
                        UpdateInterval::clamp(interval.get())
                    }),
                }) as Arc<dyn InProcessAgent>),
                Ok(Err(StartError::NotElevated)) => Err(InProcessStartError::NotElevated),
                Ok(Err(error)) => Err(InProcessStartError::Failed(error.to_string())),
                Err(error) => Err(InProcessStartError::Failed(error.to_string())),
            }
        })
    }

    impl InProcessAgent for LocalAgent {
        fn report(self: Arc<Self>) -> BoxFuture<'static, anyhow::Result<Option<WindowsReport>>> {
            Box::pin(async move { self.feed.report().await })
        }

        fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32> {
            Box::pin(async move { self.feed.act(action).await })
        }
    }
}
