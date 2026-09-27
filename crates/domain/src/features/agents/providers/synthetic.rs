use std::sync::Arc;

use app_contracts::features::agents::{
    AgentConnectionState, SignatureStatus, WindowsAgentRuntimeEvent, WindowsMachineStats,
    WindowsProcessStats, WindowsReport, WindowsReportMessage, WindowsServiceState,
    WindowsServiceStats,
};
use guinea::prelude::*;

use crate::features::settings::settings::GeneralSettings;

const VARIABLE: &str = "UNIPROC_SYNTHETIC_AGENT";
const UNIQUE_VARIABLE: &str = "UNIPROC_SYNTHETIC_UNIQUE";
const JITTER_VARIABLE: &str = "UNIPROC_SYNTHETIC_JITTER";
const DEFAULT_PROCESSES: usize = 400;
const SERVICES: usize = 250;
const TOTAL_BYTES: u64 = 32 * 1024 * 1024 * 1024;

pub fn requested() -> Option<usize> {
    let value = std::env::var(VARIABLE).ok()?;
    Some(value.parse().unwrap_or(DEFAULT_PROCESSES))
}

pub fn install(app: &mut FeatureBuilder, processes: usize) -> anyhow::Result<()> {
    let interval = GeneralSettings::new()?.update_interval();
    let executables = executables();
    let unique = std::env::var_os(UNIQUE_VARIABLE).is_some();
    let jitter: usize = std::env::var(JITTER_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    tracing::warn!(processes, unique, jitter, ?interval, "synthetic windows agent: reports are generated");

    let mut tick = 0u64;
    app.repeat(interval, move || {
        tick += 1;
        let count = if tick % 2 == 0 {
            processes
        } else {
            processes.saturating_sub(jitter)
        };
        let report = report(&executables, count, unique, tick);
        GlobalEventBus::publish(WindowsAgentRuntimeEvent {
            state: AgentConnectionState::Connected,
            latency_ms: None,
        });
        GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(report)));
    })
    .named("synthetic-windows-agent");

    Ok(())
}

fn executables() -> Vec<Arc<str>> {
    let found: Vec<Arc<str>> = std::fs::read_dir(r"C:\Windows\System32")
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "exe"))
                .take(160)
                .map(|path| path.to_string_lossy().into())
                .collect()
        })
        .unwrap_or_default();
    if found.is_empty() {
        vec![r"C:\Windows\System32\svchost.exe".into()]
    } else {
        found
    }
}

fn wave(seed: u64, tick: u64, period: u64) -> u64 {
    (seed.wrapping_mul(2_654_435_761).wrapping_add(tick * 97)) % period
}

fn report(executables: &[Arc<str>], processes: usize, unique: bool, tick: u64) -> WindowsReport {
    let processes: Vec<WindowsProcessStats> = (0..processes)
        .map(|index| {
            let seed = index as u64;
            let path = &executables[index % executables.len()];
            let file = path.rsplit('\\').next().unwrap_or_default();
            let name: Arc<str> = if unique {
                format!("{index}-{file}").into()
            } else {
                file.into()
            };
            WindowsProcessStats {
                pid: 1_000 + index as u32 * 4,
                parent_pid: 4,
                session_id: 1,
                display_name: name.clone(),
                name,
                first_arg: path.clone(),
                cpu_percent: wave(seed, tick, 400) as f32 / 40.0,
                working_set_bytes: (4_096 + wave(seed, tick, 200_000)) * 1024,
                private_working_set_bytes: (2_048 + wave(seed, tick, 150_000)) * 1024,
                disk_read_bytes: wave(seed + 1, tick, 90_000),
                disk_write_bytes: wave(seed + 2, tick, 40_000),
                net_rx_bytes: wave(seed + 3, tick, 60_000),
                net_tx_bytes: wave(seed + 4, tick, 20_000),
                is_service: index % 5 == 0,
                is_kernel_process: index < 4,
                is_windows_process: index % 3 == 0,
                signature: if index % 2 == 0 {
                    SignatureStatus::Microsoft
                } else {
                    SignatureStatus::ThirdParty
                },
                image_path: path.clone(),
                ..WindowsProcessStats::default()
            }
        })
        .collect();

    let services = (0..SERVICES)
        .map(|index| WindowsServiceStats {
            name: format!("SyntheticService{index}").into(),
            display_name: format!("Synthetic service {index}").into(),
            pid: if index % 3 == 0 { 0 } else { 1_000 + index as u32 * 4 },
            state: if index % 3 == 0 {
                WindowsServiceState::Stopped
            } else {
                WindowsServiceState::Running
            },
            load_group: format!("Group{}", index % 7).into(),
            description: format!("Generated service number {index} for measurements").into(),
            image_path: executables[index % executables.len()].clone(),
        })
        .collect();

    let used = TOTAL_BYTES / 3 + wave(7, tick, TOTAL_BYTES / 4);
    WindowsReport {
        machine: WindowsMachineStats {
            total_physical_bytes: TOTAL_BYTES,
            available_physical_bytes: TOTAL_BYTES - used,
            cpu_percent: wave(11, tick, 1_000) as f32 / 10.0,
            cpu_max_mhz: 4_800,
            cpu_current_mhz: 3_000 + wave(13, tick, 1_800),
            ..WindowsMachineStats::default()
        },
        processes,
        services,
    }
}
