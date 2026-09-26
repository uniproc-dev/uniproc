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

#[cfg(windows)]
pub use embedded::start_embedded;

#[cfg(not(windows))]
pub fn start_embedded() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>> {
    Box::pin(async { Err(InProcessStartError::Failed("the in-process agent is Windows only".into())) })
}

#[cfg(windows)]
mod embedded {
    use std::collections::HashMap;
    use std::sync::Arc;

    use app_contracts::features::agents::{
        ProcessPriority, SignatureStatus, WindowsAction, WindowsMachineStats, WindowsProcessStats,
        WindowsReport, WindowsServiceState, WindowsServiceStats,
    };
    use futures::future::BoxFuture;
    use uniproc_windows_agent::agent::Agent;
    use uniproc_windows_agent::api;
    use uniproc_windows_agent::embedded::{Embedded, StartError};

    use super::{InProcessAgent, InProcessStartError};

    pub fn start_embedded() -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>> {
        Box::pin(async {
            match tokio::task::spawn_blocking(Embedded::start).await {
                Ok(Ok(embedded)) => Ok(Arc::new(embedded) as Arc<dyn InProcessAgent>),
                Ok(Err(StartError::NotElevated)) => Err(InProcessStartError::NotElevated),
                Ok(Err(error)) => Err(InProcessStartError::Failed(error.to_string())),
                Err(error) => Err(InProcessStartError::Failed(error.to_string())),
            }
        })
    }

    impl InProcessAgent for Embedded {
        fn report(&self) -> WindowsReport {
            report(self.snapshot())
        }

        fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32> {
            Box::pin(async move {
                match Agent::Embedded(self).run(command(action)).await {
                    Ok(Ok(())) => 0,
                    Ok(Err(code)) => code,
                    Err(_) => u32::MAX,
                }
            })
        }
    }

    fn report(snapshot: api::Snapshot) -> WindowsReport {
        let infos: HashMap<u32, &api::ProcessInfo> =
            snapshot.processes.value.iter().map(|info| (info.pid, info)).collect();
        let processes = snapshot
            .metrics
            .iter()
            .filter_map(|metrics| infos.get(&metrics.pid).map(|info| process(info, metrics)))
            .collect();

        WindowsReport {
            machine: machine(&snapshot.machine),
            processes,
            services: snapshot.services.value.iter().map(service).collect(),
        }
    }

    fn machine(m: &api::MachineStats) -> WindowsMachineStats {
        WindowsMachineStats {
            total_physical_kb: m.total_physical_kb,
            available_physical_kb: m.available_physical_kb,
            used_physical_kb: m.used_physical_kb,
            cpu_percent: m.cpu_percent,
            cpu_max_mhz: m.cpu_max_mhz,
            cpu_current_mhz: m.cpu_current_mhz,
            disk_read_bytes: m.disk_read_bytes,
            disk_write_bytes: m.disk_write_bytes,
            disk_read_iops: m.disk_read_iops,
            disk_write_iops: m.disk_write_iops,
            net_rx_bytes: m.net_rx_bytes,
            net_tx_bytes: m.net_tx_bytes,
        }
    }

    fn process(info: &api::ProcessInfo, m: &api::ProcessMetrics) -> WindowsProcessStats {
        WindowsProcessStats {
            pid: info.pid,
            parent_pid: info.parent_pid,
            session_id: info.session_id,
            name: Arc::from(info.name.as_str()),
            first_arg: Arc::from(info.cmdline.first().map_or("", String::as_str)),
            package_full_name: Arc::from(info.package_full_name.as_str()),
            package_relative_app_id: Arc::from(info.package_relative_app_id.as_str()),
            cpu_percent: m.cpu_percent,
            working_set_kb: m.working_set_kb,
            private_bytes_kb: m.private_bytes_kb,
            peak_working_set_kb: m.peak_working_set_kb,
            private_working_set_kb: m.private_working_set_kb,
            disk_read_bytes: m.disk_read_bytes,
            disk_write_bytes: m.disk_write_bytes,
            disk_read_iops: m.disk_read_iops,
            disk_write_iops: m.disk_write_iops,
            net_rx_bytes: m.net_rx_bytes,
            net_tx_bytes: m.net_tx_bytes,
            is_service: info.is_service,
            is_kernel_process: info.is_kernel_process,
            is_windows_process: info.is_windows_process,
            signature: signature(info.signature),
            image_path: Arc::from(info.image_path.as_str()),
            display_name: Arc::from(info.display_name.as_str()),
            console_host_pid: info.console_host_pid,
        }
    }

    fn service(s: &api::ServiceStats) -> WindowsServiceStats {
        WindowsServiceStats {
            name: Arc::from(s.name.as_str()),
            display_name: Arc::from(s.display_name.as_str()),
            pid: s.pid,
            state: service_state(s.state),
            load_group: Arc::from(s.load_group.as_str()),
            description: Arc::from(s.description.as_str()),
            image_path: Arc::from(s.image_path.as_str()),
        }
    }

    fn signature(status: api::SignatureStatus) -> SignatureStatus {
        match status {
            api::SignatureStatus::Unknown => SignatureStatus::Unknown,
            api::SignatureStatus::Unsigned => SignatureStatus::Unsigned,
            api::SignatureStatus::Microsoft => SignatureStatus::Microsoft,
            api::SignatureStatus::ThirdParty => SignatureStatus::ThirdParty,
        }
    }

    fn service_state(state: api::ServiceState) -> WindowsServiceState {
        match state {
            api::ServiceState::Unknown => WindowsServiceState::Unknown,
            api::ServiceState::Stopped => WindowsServiceState::Stopped,
            api::ServiceState::StartPending => WindowsServiceState::StartPending,
            api::ServiceState::StopPending => WindowsServiceState::StopPending,
            api::ServiceState::Running => WindowsServiceState::Running,
            api::ServiceState::ContinuePending => WindowsServiceState::ContinuePending,
            api::ServiceState::PausePending => WindowsServiceState::PausePending,
            api::ServiceState::Paused => WindowsServiceState::Paused,
        }
    }

    fn priority(priority: ProcessPriority) -> api::ProcessPriority {
        match priority {
            ProcessPriority::Idle => api::ProcessPriority::Idle,
            ProcessPriority::BelowNormal => api::ProcessPriority::BelowNormal,
            ProcessPriority::Normal => api::ProcessPriority::Normal,
            ProcessPriority::AboveNormal => api::ProcessPriority::AboveNormal,
            ProcessPriority::High => api::ProcessPriority::High,
            ProcessPriority::Realtime => api::ProcessPriority::Realtime,
        }
    }

    fn command(action: WindowsAction) -> api::Command {
        match action {
            WindowsAction::Kill { pid } => api::Command::Kill { pid },
            WindowsAction::Suspend { pid } => api::Command::Suspend { pid },
            WindowsAction::Resume { pid } => api::Command::Resume { pid },
            WindowsAction::SetPriority { pid, priority: p } => {
                api::Command::SetPriority { pid, priority: priority(p) }
            }
            WindowsAction::SetAffinity { pid, mask } => api::Command::SetAffinity { pid, mask },
            WindowsAction::ServiceStart { name } => api::Command::ServiceStart { name },
            WindowsAction::ServiceStop { name } => api::Command::ServiceStop { name },
            WindowsAction::ServicePause { name } => api::Command::ServicePause { name },
            WindowsAction::ServiceResume { name } => api::Command::ServiceResume { name },
            WindowsAction::ServiceRestart { name } => api::Command::ServiceRestart { name },
        }
    }
}
