use std::sync::Arc;

use app_contracts::features::agents::{WindowsAction, WindowsReport};
use futures::future::BoxFuture;

pub enum InProcessStartError {
    NotElevated,
    Failed(String),
}

pub trait InProcessAgent: Send + Sync + 'static {
    fn report(&self) -> WindowsReport;
    fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32>;
}

pub type InProcessStart =
    fn() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>>;

pub use local::start_local;

mod local {
    use std::sync::{Arc, Mutex, PoisonError};

    use app_contracts::features::agents::{WindowsAction, WindowsReport};
    use futures::future::BoxFuture;
    use uniproc_windows_agent::agent::Agent;
    use uniproc_windows_agent::local::{Local, StartError};

    use super::{InProcessAgent, InProcessStartError};
    use crate::features::agents::windows_report::{self, Reports};

    struct LocalAgent {
        local: Arc<Local>,
        reports: Mutex<Reports>,
    }

    pub fn start_local() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>> {
        Box::pin(async {
            match tokio::task::spawn_blocking(Local::start).await {
                Ok(Ok(local)) => Ok(Arc::new(LocalAgent {
                    local: Arc::new(local),
                    reports: Mutex::default(),
                }) as Arc<dyn InProcessAgent>),
                Ok(Err(StartError::NotElevated)) => Err(InProcessStartError::NotElevated),
                Ok(Err(error)) => Err(InProcessStartError::Failed(error.to_string())),
                Err(error) => Err(InProcessStartError::Failed(error.to_string())),
            }
        })
    }

    impl InProcessAgent for LocalAgent {
        fn report(&self) -> WindowsReport {
            let snapshot = self.local.snapshot();
            self.reports
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .report(&snapshot)
        }

        fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32> {
            Box::pin(async move {
                let agent = Agent::Local(self.local.clone());
                windows_report::code(agent.run(windows_report::command(action)).await)
            })
        }
    }
}
