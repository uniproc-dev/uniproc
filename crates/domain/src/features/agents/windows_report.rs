use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agents::{
    Architecture, DpiAwareness, ExtendedCfg, GpuEngineId, GpuEngineKind, IoPriority, Isolation, Mitigations,
    ProcessPriority, ProcessRunState, SignatureStatus, StackProtection, UacVirtualization, WindowsAction,
    WindowsGpu, WindowsGpuEngine, WindowsMachineStats, WindowsProcessStats, WindowsProcessorStats, WindowsReport,
    WindowsServiceState, WindowsServiceStats,
};
use uniproc_windows_agent::api::{self, MachineMetrics, MetricSpec, ProcessMetrics};

type Key = (u32, u64);
type EngineKey = (u64, u32);

pub fn spec(interval: Duration) -> MetricSpec {
    MetricSpec {
        interval,
        processes: ProcessMetrics::all(),
        machine: MachineMetrics::all(),
    }
}

#[derive(Default)]
pub struct Reports {
    passports: HashMap<Key, WindowsProcessStats>,
    states: HashMap<Key, ProcessRunState>,
    services: Vec<WindowsServiceStats>,
    cpu_times: HashMap<Key, u64>,
    gpu_times: HashMap<(Key, EngineKey), u64>,
    engine_times: HashMap<EngineKey, u64>,
    machine_cpu: Option<api::MachineCpu>,
    processors: Option<Arc<[api::MachineProcessor]>>,
    sampled_at: Option<u64>,
}

struct ProcessGpu {
    percent: f32,
    engine: GpuEngineId,
}

impl Reports {
    #[tracing::instrument(skip_all, level = "debug", fields(full = update.changes.full, passports = update.changes.passports.len(), left = update.changes.left.len()))]
    pub fn report(&mut self, update: &api::Update) -> WindowsReport {
        let api::Update { snapshot, sample, changes } = update;
        self.keep_passports(&snapshot.processes.value, changes);
        self.keep_states(&snapshot.states.value, changes);
        if changes.full || changes.services {
            self.services = snapshot.services.value.iter().map(service).collect();
        }

        let elapsed = machine_time(self.machine_cpu, sample.machine.cpu);
        let wall = self.sampled_at.map_or(0, |before| sample.sampled_at.saturating_sub(before));
        let gpu = self.process_gpu(sample, wall);
        let columns = &sample.columns;
        let mut cpu_times = HashMap::with_capacity(sample.pids.len());
        let processes = sample
            .pids
            .iter()
            .zip(sample.sequence_numbers.iter())
            .enumerate()
            .filter_map(|(row, (&pid, &sequence_number))| {
                let key = (pid, sequence_number);
                let cpu_time = read(&columns.cpu_user_time, row)
                    .zip(read(&columns.cpu_kernel_time, row))
                    .map(|(user, kernel)| user.saturating_add(kernel));
                let before = self.cpu_times.get(&key).copied();
                if let Some(cpu_time) = cpu_time {
                    cpu_times.insert(key, cpu_time);
                }
                let passport = self.passports.get(&key)?;
                let gpu = gpu.get(&row);
                Some(WindowsProcessStats {
                    cpu_percent: before
                        .zip(cpu_time)
                        .map_or(0.0, |(before, now)| share(now.saturating_sub(before), elapsed)),
                    cpu_cycles: at(&columns.cpu_cycles, row),
                    working_set_bytes: at(&columns.working_set, row),
                    commit_bytes: at(&columns.commit, row),
                    peak_working_set_bytes: at(&columns.peak_working_set, row),
                    private_working_set_bytes: at(&columns.private_working_set, row),
                    peak_commit_bytes: at(&columns.peak_commit, row),
                    virtual_size_bytes: at(&columns.virtual_size, row),
                    peak_virtual_size_bytes: at(&columns.peak_virtual_size, row),
                    paged_pool_bytes: at(&columns.paged_pool, row),
                    peak_paged_pool_bytes: at(&columns.peak_paged_pool, row),
                    non_paged_pool_bytes: at(&columns.non_paged_pool, row),
                    peak_non_paged_pool_bytes: at(&columns.peak_non_paged_pool, row),
                    page_faults: wrapping(&columns.page_faults, row),
                    hard_faults: wrapping(&columns.hard_faults, row),
                    handles: at32(&columns.handles, row),
                    threads: at32(&columns.threads, row),
                    peak_threads: at32(&columns.peak_threads, row),
                    context_switches: at(&columns.context_switches, row),
                    user_objects: at32(&columns.user_objects, row),
                    gdi_objects: at32(&columns.gdi_objects, row),
                    io_read_ops: at(&columns.io_read_ops, row),
                    io_write_ops: at(&columns.io_write_ops, row),
                    io_other_ops: at(&columns.io_other_ops, row),
                    io_read_bytes: at(&columns.io_read_bytes, row),
                    io_write_bytes: at(&columns.io_write_bytes, row),
                    io_other_bytes: at(&columns.io_other_bytes, row),
                    disk_read_bytes: at(&columns.disk_read_bytes, row),
                    disk_write_bytes: at(&columns.disk_write_bytes, row),
                    disk_read_iops: at(&columns.disk_read_ops, row),
                    disk_write_iops: at(&columns.disk_write_ops, row),
                    disk_flush_ops: at(&columns.disk_flush_ops, row),
                    net_rx_bytes: at(&columns.net_rx_bytes, row),
                    net_tx_bytes: at(&columns.net_tx_bytes, row),
                    gpu_percent: gpu.map_or(0.0, |gpu| gpu.percent),
                    gpu_engine: gpu.map(|gpu| gpu.engine),
                    gpu_dedicated_bytes: at(&columns.gpu_dedicated, row),
                    gpu_shared_bytes: at(&columns.gpu_shared, row),
                    state: self.states.get(&key).copied().unwrap_or_default(),
                    ..passport.clone()
                })
            })
            .collect();
        self.cpu_times = cpu_times;

        let machine = self.machine(&sample.machine, wall);
        self.machine_cpu = sample.machine.cpu;
        self.processors = sample.machine.processors.clone();
        self.sampled_at = Some(sample.sampled_at);

        WindowsReport {
            machine,
            processes,
            services: self.services.clone(),
        }
    }

    fn process_gpu(&mut self, sample: &api::Sample, wall: u64) -> HashMap<usize, ProcessGpu> {
        let Some(engines) = sample.gpu_engines.as_deref() else {
            self.gpu_times.clear();
            return HashMap::new();
        };
        let mut times = HashMap::with_capacity(engines.len());
        let mut busiest: HashMap<usize, ProcessGpu> = HashMap::new();
        for entry in engines {
            let row = entry.row as usize;
            let (Some(&pid), Some(&sequence_number)) = (sample.pids.get(row), sample.sequence_numbers.get(row)) else {
                continue;
            };
            let key = ((pid, sequence_number), (entry.adapter_luid, entry.engine));
            let before = self.gpu_times.get(&key).copied();
            times.insert(key, entry.running_time);
            let percent = before.map_or(0.0, |before| share(entry.running_time.wrapping_sub(before), wall));
            let engine = GpuEngineId { adapter_luid: entry.adapter_luid, ordinal: entry.engine };
            match busiest.get(&row) {
                Some(kept) if kept.percent >= percent => {}
                _ => {
                    busiest.insert(row, ProcessGpu { percent, engine });
                }
            }
        }
        self.gpu_times = times;
        busiest
    }

    fn keep_states(&mut self, states: &api::ProcessStates, changes: &api::Changes) {
        if changes.full {
            self.states = states.states.iter().map(|state| (state_key(state), run_state(state))).collect();
            return;
        }
        for left in &changes.left {
            self.states.remove(left);
        }
        if changes.states.is_empty() {
            return;
        }
        let changed: HashSet<Key> = changes.states.iter().copied().collect();
        for state in states.states.iter().filter(|state| changed.contains(&state_key(state))) {
            self.states.insert(state_key(state), run_state(state));
        }
    }

    fn machine(&mut self, sample: &api::MachineSample, wall: u64) -> WindowsMachineStats {
        let cpu = sample.cpu.unwrap_or_default();
        let memory = sample.memory.unwrap_or_default();
        let disk = sample.disk.unwrap_or_default();
        let network = sample.network.unwrap_or_default();
        let whole = breakdown(self.machine_cpu.map(Times::of_machine), sample.cpu.map(Times::of_machine));
        let processors = match (self.processors.as_deref(), sample.processors.as_deref()) {
            (Some(before), Some(now)) if before.len() == now.len() => before
                .iter()
                .zip(now)
                .map(|(before, now)| breakdown(Some(Times::of_processor(before)), Some(Times::of_processor(now))))
                .collect(),
            (_, Some(now)) => now.iter().map(|_| WindowsProcessorStats::default()).collect(),
            _ => Arc::from([]),
        };
        WindowsMachineStats {
            total_physical_bytes: memory.total_physical,
            available_physical_bytes: memory.available_physical,
            commit_limit_bytes: memory.commit_limit,
            committed_bytes: memory.committed,
            cpu_percent: busy(self.machine_cpu, sample.cpu),
            cpu_user_percent: whole.user_percent,
            cpu_kernel_percent: whole.kernel_percent,
            cpu_interrupt_percent: whole.interrupt_percent,
            cpu_dpc_percent: whole.dpc_percent,
            cpu_max_mhz: cpu.max_mhz.into(),
            cpu_current_mhz: cpu.current_mhz.into(),
            processors,
            disk_read_bytes: disk.read_bytes,
            disk_write_bytes: disk.write_bytes,
            disk_read_iops: disk.read_ops,
            disk_write_iops: disk.write_ops,
            net_rx_bytes: network.rx_bytes,
            net_tx_bytes: network.tx_bytes,
            gpus: self.gpus(sample.gpus.as_deref(), wall),
        }
    }

    fn gpus(&mut self, adapters: Option<&[api::GpuAdapter]>, wall: u64) -> Arc<[WindowsGpu]> {
        let Some(adapters) = adapters else {
            self.engine_times.clear();
            return Arc::from([]);
        };
        let mut times = HashMap::new();
        let gpus = adapters
            .iter()
            .map(|adapter| {
                let engines = adapter
                    .engines
                    .iter()
                    .map(|engine| {
                        let key = (adapter.luid, engine.ordinal);
                        let before = self.engine_times.get(&key).copied();
                        times.insert(key, engine.running_time);
                        WindowsGpuEngine {
                            ordinal: engine.ordinal,
                            kind: engine_kind(engine.kind),
                            name: Arc::from(engine.name.as_str()),
                            busy_percent: before
                                .map_or(0.0, |before| share(engine.running_time.wrapping_sub(before), wall)),
                            frequency_hz: engine.frequency,
                            max_frequency_hz: engine.max_frequency,
                        }
                    })
                    .collect();
                WindowsGpu {
                    luid: adapter.luid,
                    name: Arc::from(adapter.name.as_str()),
                    dedicated_limit_bytes: adapter.dedicated_limit,
                    dedicated_usage_bytes: adapter.dedicated_usage,
                    shared_limit_bytes: adapter.shared_limit,
                    shared_usage_bytes: adapter.shared_usage,
                    temperature_celsius: (adapter.temperature > 0).then(|| adapter.temperature as f32 / 10.0),
                    fan_rpm: adapter.fan_rpm,
                    power_percent: adapter.power as f32 / 10.0,
                    memory_frequency_hz: adapter.memory_frequency,
                    engines,
                }
            })
            .collect();
        self.engine_times = times;
        gpus
    }

    fn keep_passports(&mut self, processes: &[api::ProcessInfo], changes: &api::Changes) {
        if changes.full {
            self.passports = processes.iter().map(|info| (key(info), passport(info))).collect();
            return;
        }
        for left in &changes.left {
            self.passports.remove(left);
        }
        if changes.passports.is_empty() {
            return;
        }
        let changed: HashSet<Key> = changes.passports.iter().copied().collect();
        for info in processes.iter().filter(|info| changed.contains(&key(info))) {
            self.passports.insert(key(info), passport(info));
        }
    }
}

fn key(info: &api::ProcessInfo) -> Key {
    (info.pid, info.sequence_number)
}

fn read(column: &Option<Arc<[u64]>>, row: usize) -> Option<u64> {
    column
        .as_deref()
        .and_then(|values| values.get(row))
        .copied()
        .filter(|value| *value != api::NO_DATA_U64)
}

fn at(column: &Option<Arc<[u64]>>, row: usize) -> u64 {
    read(column, row).unwrap_or_default()
}

fn at32(column: &Option<Arc<[u32]>>, row: usize) -> u32 {
    column
        .as_deref()
        .and_then(|values| values.get(row))
        .copied()
        .filter(|value| *value != api::NO_DATA_U32)
        .unwrap_or_default()
}

fn wrapping(column: &Option<Arc<[u32]>>, row: usize) -> u32 {
    column.as_deref().and_then(|values| values.get(row)).copied().unwrap_or_default()
}

fn state_key(state: &api::ProcessState) -> Key {
    (state.pid, state.sequence_number)
}

#[derive(Clone, Copy)]
struct Times {
    idle: u64,
    kernel: u64,
    user: u64,
    interrupt: u64,
    dpc: u64,
}

impl Times {
    fn of_machine(cpu: api::MachineCpu) -> Self {
        Self {
            idle: cpu.idle_time,
            kernel: cpu.kernel_time,
            user: cpu.user_time,
            interrupt: cpu.interrupt_time,
            dpc: cpu.dpc_time,
        }
    }

    fn of_processor(processor: &api::MachineProcessor) -> Self {
        Self {
            idle: processor.idle_time,
            kernel: processor.kernel_time,
            user: processor.user_time,
            interrupt: processor.interrupt_time,
            dpc: processor.dpc_time,
        }
    }
}

fn breakdown(before: Option<Times>, now: Option<Times>) -> WindowsProcessorStats {
    let (Some(before), Some(now)) = (before, now) else {
        return WindowsProcessorStats::default();
    };
    let kernel = now.kernel.saturating_sub(before.kernel);
    let user = now.user.saturating_sub(before.user);
    let idle = now.idle.saturating_sub(before.idle);
    let whole = kernel + user;
    WindowsProcessorStats {
        busy_percent: share(whole.saturating_sub(idle), whole),
        user_percent: share(user, whole),
        kernel_percent: share(kernel.saturating_sub(idle), whole),
        interrupt_percent: share(now.interrupt.saturating_sub(before.interrupt), whole),
        dpc_percent: share(now.dpc.saturating_sub(before.dpc), whole),
    }
}

fn total(cpu: &api::MachineCpu) -> u64 {
    cpu.kernel_time + cpu.user_time
}

fn machine_time(before: Option<api::MachineCpu>, now: Option<api::MachineCpu>) -> u64 {
    match (before, now) {
        (Some(before), Some(now)) => total(&now).saturating_sub(total(&before)),
        _ => 0,
    }
}

fn share(part: u64, whole: u64) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64 / whole as f64 * 100.0).min(100.0) as f32
}

fn busy(before: Option<api::MachineCpu>, now: Option<api::MachineCpu>) -> f32 {
    let (Some(before), Some(now)) = (before, now) else {
        return 0.0;
    };
    let whole = total(&now).saturating_sub(total(&before));
    let idle = now.idle_time.saturating_sub(before.idle_time);
    share(whole.saturating_sub(idle), whole)
}

fn passport(info: &api::ProcessInfo) -> WindowsProcessStats {
    WindowsProcessStats {
        pid: info.pid,
        parent_pid: info.parent_pid,
        session_id: info.session_id,
        name: Arc::from(info.name.as_str()),
        first_arg: Arc::from(info.cmdline.first().map_or("", String::as_str)),
        package_full_name: Arc::from(info.package_full_name.as_str()),
        package_relative_app_id: Arc::from(info.package_relative_app_id.as_str()),
        is_service: info.is_service,
        is_kernel_process: info.is_kernel_process,
        is_windows_process: info.is_windows_process,
        signature: signature(info.signature),
        image_path: Arc::from(info.image_path.as_str()),
        display_name: Arc::from(info.display_name.as_str()),
        console_host_pid: info.console_host_pid,
        start_time: info.start_time,
        user: Arc::from(info.user.as_str()),
        architecture: architecture(info.architecture),
        elevated: info.elevated,
        uac_virtualization: uac_virtualization(info.uac_virtualization),
        isolation: isolation(info.isolation),
        dpi_awareness: dpi_awareness(info.dpi_awareness),
        mitigations: info.mitigations.map(mitigations),
        publisher: Arc::from(info.publisher.as_str()),
        ..WindowsProcessStats::default()
    }
}

fn run_state(state: &api::ProcessState) -> ProcessRunState {
    ProcessRunState {
        suspended: state.suspended,
        efficiency_mode: state.efficiency_mode,
        base_priority: state.base_priority.map(priority_class),
        power_throttling: state.power_throttling,
        job_object_id: state.job_object_id,
        io_priority: io_priority(state.io_priority),
    }
}

fn architecture(architecture: api::Architecture) -> Architecture {
    match architecture {
        api::Architecture::Unknown => Architecture::Unknown,
        api::Architecture::X86 => Architecture::X86,
        api::Architecture::X64 => Architecture::X64,
        api::Architecture::Arm => Architecture::Arm,
        api::Architecture::Arm64 => Architecture::Arm64,
        api::Architecture::Arm64X86Compatible => Architecture::Arm64X86Compatible,
        api::Architecture::Arm64X64Compatible => Architecture::Arm64X64Compatible,
    }
}

fn uac_virtualization(uac: api::UacVirtualization) -> UacVirtualization {
    match uac {
        api::UacVirtualization::Unknown => UacVirtualization::Unknown,
        api::UacVirtualization::NotAllowed => UacVirtualization::NotAllowed,
        api::UacVirtualization::Disabled => UacVirtualization::Disabled,
        api::UacVirtualization::Enabled => UacVirtualization::Enabled,
    }
}

fn isolation(isolation: api::Isolation) -> Isolation {
    match isolation {
        api::Isolation::Unknown => Isolation::Unknown,
        api::Isolation::None => Isolation::None,
        api::Isolation::AppContainer => Isolation::AppContainer,
        api::Isolation::Uwp => Isolation::Uwp,
        api::Isolation::Silo => Isolation::Silo,
    }
}

fn dpi_awareness(dpi: api::DpiAwareness) -> DpiAwareness {
    match dpi {
        api::DpiAwareness::Unknown => DpiAwareness::Unknown,
        api::DpiAwareness::Unaware => DpiAwareness::Unaware,
        api::DpiAwareness::System => DpiAwareness::System,
        api::DpiAwareness::PerMonitor => DpiAwareness::PerMonitor,
        api::DpiAwareness::PerMonitorV2 => DpiAwareness::PerMonitorV2,
        api::DpiAwareness::UnawareGdiScaled => DpiAwareness::UnawareGdiScaled,
    }
}

fn mitigations(mitigations: api::Mitigations) -> Mitigations {
    Mitigations {
        dep: mitigations.dep,
        stack_protection: match mitigations.stack_protection {
            api::StackProtection::Unknown => StackProtection::Unknown,
            api::StackProtection::Off => StackProtection::Off,
            api::StackProtection::Compatible => StackProtection::Compatible,
            api::StackProtection::Strict => StackProtection::Strict,
            api::StackProtection::CompatibleAudit => StackProtection::CompatibleAudit,
            api::StackProtection::StrictAudit => StackProtection::StrictAudit,
        },
        extended_cfg: match mitigations.extended_cfg {
            api::ExtendedCfg::Unknown => ExtendedCfg::Unknown,
            api::ExtendedCfg::Off => ExtendedCfg::Off,
            api::ExtendedCfg::Audit => ExtendedCfg::Audit,
            api::ExtendedCfg::On => ExtendedCfg::On,
        },
    }
}

fn io_priority(priority: api::IoPriority) -> IoPriority {
    match priority {
        api::IoPriority::Unknown => IoPriority::Unknown,
        api::IoPriority::VeryLow => IoPriority::VeryLow,
        api::IoPriority::Low => IoPriority::Low,
        api::IoPriority::Normal => IoPriority::Normal,
        api::IoPriority::High => IoPriority::High,
        api::IoPriority::Critical => IoPriority::Critical,
    }
}

fn priority_class(priority: api::ProcessPriority) -> ProcessPriority {
    match priority {
        api::ProcessPriority::Idle => ProcessPriority::Idle,
        api::ProcessPriority::BelowNormal => ProcessPriority::BelowNormal,
        api::ProcessPriority::Normal => ProcessPriority::Normal,
        api::ProcessPriority::AboveNormal => ProcessPriority::AboveNormal,
        api::ProcessPriority::High => ProcessPriority::High,
        api::ProcessPriority::Realtime => ProcessPriority::Realtime,
    }
}

fn engine_kind(kind: api::GpuEngineKind) -> GpuEngineKind {
    match kind {
        api::GpuEngineKind::Other => GpuEngineKind::Other,
        api::GpuEngineKind::ThreeD => GpuEngineKind::ThreeD,
        api::GpuEngineKind::VideoDecode => GpuEngineKind::VideoDecode,
        api::GpuEngineKind::VideoEncode => GpuEngineKind::VideoEncode,
        api::GpuEngineKind::VideoProcessing => GpuEngineKind::VideoProcessing,
        api::GpuEngineKind::SceneAssembly => GpuEngineKind::SceneAssembly,
        api::GpuEngineKind::Copy => GpuEngineKind::Copy,
        api::GpuEngineKind::Overlay => GpuEngineKind::Overlay,
        api::GpuEngineKind::Crypto => GpuEngineKind::Crypto,
        api::GpuEngineKind::VideoCodec => GpuEngineKind::VideoCodec,
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

pub fn command(action: WindowsAction) -> api::Command {
    match action {
        WindowsAction::Kill { pid } => api::Command::Kill { pid },
        WindowsAction::Suspend { pid } => api::Command::Suspend { pid },
        WindowsAction::Resume { pid } => api::Command::Resume { pid },
        WindowsAction::SetPriority { pid, priority: p } => api::Command::SetPriority { pid, priority: priority(p) },
        WindowsAction::SetAffinity { pid, mask } => api::Command::SetAffinity { pid, mask },
        WindowsAction::ServiceStart { name } => api::Command::ServiceStart { name },
        WindowsAction::ServiceStop { name } => api::Command::ServiceStop { name },
        WindowsAction::ServicePause { name } => api::Command::ServicePause { name },
        WindowsAction::ServiceResume { name } => api::Command::ServiceResume { name },
        WindowsAction::ServiceRestart { name } => api::Command::ServiceRestart { name },
    }
}

pub fn code(result: anyhow::Result<api::CommandResult>) -> u32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(code)) => code,
        Err(_) => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequence(pid: u32) -> u64 {
        u64::from(pid) * 10
    }

    fn info(pid: u32, name: &str, cmdline: &[&str]) -> api::ProcessInfo {
        api::ProcessInfo {
            pid,
            sequence_number: sequence(pid),
            name: name.into(),
            cmdline: cmdline.iter().map(|arg| arg.to_string()).collect(),
            ..api::ProcessInfo::default()
        }
    }

    fn update(snapshot: api::Snapshot, sample: api::Sample, changes: api::Changes) -> api::Update {
        api::Update { snapshot, sample, changes }
    }

    fn full(snapshot: api::Snapshot, sample: api::Sample) -> api::Update {
        update(snapshot, sample, api::Changes { full: true, ..api::Changes::default() })
    }

    fn same(snapshot: api::Snapshot, sample: api::Sample) -> api::Update {
        update(snapshot, sample, api::Changes::default())
    }

    fn key(pid: u32) -> (u32, u64) {
        (pid, sequence(pid))
    }

    fn snapshot(etag: u64, infos: Vec<api::ProcessInfo>) -> api::Snapshot {
        api::Snapshot {
            services: api::Tagged { etag: 1, value: Arc::from([]) },
            processes: api::Tagged { etag, value: Arc::from(infos) },
            states: api::Tagged { etag: 1, value: api::ProcessStates::default() },
        }
    }

    struct Row {
        pid: u32,
        sequence_number: u64,
        cpu_time: u64,
        disk_read_bytes: u64,
    }

    fn row(pid: u32, cpu_time: u64) -> Row {
        Row {
            pid,
            sequence_number: sequence(pid),
            cpu_time,
            disk_read_bytes: 0,
        }
    }

    fn sample(rows: &[Row], machine_time: u64, idle_time: u64) -> api::Sample {
        api::Sample {
            pids: rows.iter().map(|r| r.pid).collect(),
            sequence_numbers: rows.iter().map(|r| r.sequence_number).collect(),
            columns: api::Columns {
                cpu_user_time: Some(rows.iter().map(|r| r.cpu_time).collect()),
                cpu_kernel_time: Some(rows.iter().map(|_| 0).collect()),
                disk_read_bytes: Some(rows.iter().map(|r| r.disk_read_bytes).collect()),
                ..api::Columns::default()
            },
            machine: api::MachineSample {
                cpu: Some(api::MachineCpu {
                    kernel_time: machine_time,
                    idle_time,
                    ..api::MachineCpu::default()
                }),
                ..api::MachineSample::default()
            },
            ..api::Sample::default()
        }
    }

    #[test]
    fn a_passport_keeps_the_first_argument_and_the_console_host() {
        let mut console = info(7, "app.exe", &[r"C:\app.exe", "--flag"]);
        console.console_host_pid = 40;
        let report = Reports::default().report(&full(
            snapshot(1, vec![console, info(8, "idle", &[])]),
            sample(&[row(7, 0), row(8, 0)], 0, 0),
        ));

        assert_eq!(&*report.processes[0].first_arg, r"C:\app.exe");
        assert_eq!(report.processes[0].console_host_pid, 40);
        assert_eq!(&*report.processes[1].first_arg, "");
    }

    #[test]
    fn rows_join_their_passports_by_pid_and_sequence_number() {
        let mut read = row(9, 0);
        read.disk_read_bytes = 100;
        let report = Reports::default().report(&full(
            snapshot(1, vec![info(7, "a.exe", &[]), info(9, "b.exe", &[])]),
            sample(&[read, row(7, 0)], 0, 0),
        ));

        let by_pid = |pid: u32| report.processes.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(&*by_pid(9).name, "b.exe");
        assert_eq!(by_pid(9).disk_read_bytes, 100);
        assert_eq!(&*by_pid(7).name, "a.exe");
    }

    #[test]
    fn a_row_without_its_passport_is_dropped_rather_than_named_wrong() {
        let mut reused = row(8, 0);
        reused.sequence_number += 1;
        let report = Reports::default().report(&full(
            snapshot(1, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), reused, row(99, 0)], 0, 0),
        ));

        assert_eq!(report.processes.iter().map(|r| r.pid).collect::<Vec<_>>(), vec![7]);
    }

    #[test]
    fn an_unchanged_process_keeps_its_strings_across_list_changes() {
        let mut reports = Reports::default();
        let first = reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let second = reports.report(&update(
            snapshot(2, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), row(8, 0)], 0, 0),
            api::Changes { passports: vec![key(8)], states: vec![key(8)], ..api::Changes::default() },
        ));

        assert!(Arc::ptr_eq(&first.processes[0].name, &second.processes[0].name));
        assert_eq!(&*second.processes[1].name, "b.exe");
    }

    #[test]
    fn a_changed_passport_is_read_again() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let mut enriched = info(7, "a.exe", &[]);
        enriched.display_name = "App".into();
        let report = reports.report(&update(
            snapshot(2, vec![enriched]),
            sample(&[row(7, 0)], 0, 0),
            api::Changes { passports: vec![key(7)], ..api::Changes::default() },
        ));

        assert_eq!(&*report.processes[0].display_name, "App");
    }

    #[test]
    fn a_reused_pid_gets_the_new_process_strings() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let mut reborn = info(7, "b.exe", &[]);
        reborn.sequence_number += 1;
        let mut reused = row(7, 0);
        reused.sequence_number += 1;
        let report = reports.report(&update(
            snapshot(2, vec![reborn]),
            sample(&[reused], 0, 0),
            api::Changes {
                passports: vec![(7, sequence(7) + 1)],
                left: vec![key(7)],
                ..api::Changes::default()
            },
        ));

        assert_eq!(&*report.processes[0].name, "b.exe");
        assert_eq!(reports.passports.len(), 1);
    }

    #[test]
    fn services_are_read_again_only_when_they_changed() {
        let mut reports = Reports::default();
        let mut list = snapshot(1, vec![]);
        list.services.value = Arc::from([api::ServiceStats { name: "svc".into(), ..api::ServiceStats::default() }]);
        let first = reports.report(&full(list.clone(), sample(&[], 0, 0)));
        let kept = reports.report(&same(list.clone(), sample(&[], 0, 0)));
        list.services.value = Arc::from([]);
        let changed = reports.report(&update(
            list,
            sample(&[], 0, 0),
            api::Changes { services: true, ..api::Changes::default() },
        ));

        assert_eq!(first.services.len(), 1);
        assert!(Arc::ptr_eq(&first.services[0].name, &kept.services[0].name));
        assert!(changed.services.is_empty());
    }

    #[test]
    fn cpu_is_the_share_of_machine_time_between_two_samples() {
        let mut reports = Reports::default();
        let list = snapshot(1, vec![info(7, "a.exe", &[])]);
        let first = reports.report(&full(list.clone(), sample(&[row(7, 1_000)], 10_000, 4_000)));
        let second = reports.report(&same(list, sample(&[row(7, 1_250)], 11_000, 4_600)));

        assert_eq!(first.processes[0].cpu_percent, 0.0);
        assert_eq!(first.machine.cpu_percent, 0.0);
        assert_eq!(second.processes[0].cpu_percent, 25.0);
        assert_eq!(second.machine.cpu_percent, 40.0);
    }

    #[test]
    fn a_row_marked_as_no_data_reads_as_nothing_rather_than_the_maximum() {
        let mut reports = Reports::default();
        let list = snapshot(1, vec![info(7, "a.exe", &[])]);
        let mut unread = sample(&[row(7, api::NO_DATA_U64)], 10_000, 0);
        unread.columns.working_set = Some(Arc::from([api::NO_DATA_U64]));
        let first = reports.report(&full(list.clone(), unread));
        let second = reports.report(&same(list, sample(&[row(7, 500)], 11_000, 0)));

        assert_eq!(first.processes[0].working_set_bytes, 0);
        assert_eq!(second.processes[0].cpu_percent, 0.0);
    }

    fn engine(row: u32, ordinal: u32, running_time: u64) -> api::ProcessGpuEngine {
        api::ProcessGpuEngine { row, adapter_luid: 5, engine: ordinal, running_time }
    }

    fn with_gpu(mut sample: api::Sample, sampled_at: u64, engines: Vec<api::ProcessGpuEngine>) -> api::Sample {
        sample.sampled_at = sampled_at;
        sample.gpu_engines = Some(Arc::from(engines));
        sample
    }

    #[test]
    fn a_process_gpu_is_its_busiest_engine_over_the_wall_time() {
        let mut reports = Reports::default();
        let list = snapshot(1, vec![info(7, "game.exe", &[])]);
        reports.report(&full(list.clone(), with_gpu(sample(&[row(7, 0)], 0, 0), 1_000, vec![engine(0, 0, 0), engine(0, 3, 0)])));
        let report = reports.report(&same(
            list,
            with_gpu(sample(&[row(7, 0)], 0, 0), 2_000, vec![engine(0, 0, 300), engine(0, 3, 600)]),
        ));

        let game = &report.processes[0];
        assert_eq!(game.gpu_percent, 60.0);
        assert_eq!(game.gpu_engine, Some(GpuEngineId { adapter_luid: 5, ordinal: 3 }));
    }

    #[test]
    fn a_process_without_gpu_engines_shows_no_gpu() {
        let report = Reports::default().report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));

        assert_eq!(report.processes[0].gpu_percent, 0.0);
        assert_eq!(report.processes[0].gpu_engine, None);
    }

    #[test]
    fn a_gpu_engine_is_busy_for_its_share_of_the_wall_time() {
        let adapter = |running_time: u64| api::GpuAdapter {
            luid: 5,
            name: "RTX".into(),
            temperature: 455,
            power: 125,
            engines: Arc::from([api::GpuEngine { ordinal: 0, running_time, ..api::GpuEngine::default() }]),
            ..api::GpuAdapter::default()
        };
        let at = |sampled_at: u64, running_time: u64| {
            let mut sample = sample(&[], 0, 0);
            sample.sampled_at = sampled_at;
            sample.machine.gpus = Some(Arc::from([adapter(running_time)]));
            sample
        };
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![]), at(1_000, 100)));
        let report = reports.report(&same(snapshot(1, vec![]), at(3_000, 600)));

        let gpu = &report.machine.gpus[0];
        assert_eq!(gpu.engines[0].busy_percent, 25.0);
        assert_eq!(gpu.temperature_celsius, Some(45.5));
        assert_eq!(gpu.power_percent, 12.5);
    }

    #[test]
    fn each_processor_gets_its_own_share() {
        let processors = |busy: u64| -> Arc<[api::MachineProcessor]> {
            Arc::from([
                api::MachineProcessor { kernel_time: 1_000, idle_time: 1_000 - busy, ..api::MachineProcessor::default() },
                api::MachineProcessor { kernel_time: 1_000, idle_time: 1_000, ..api::MachineProcessor::default() },
            ])
        };
        let at = |kernel: u64, busy: u64| {
            let mut sample = sample(&[], kernel, 0);
            sample.machine.processors = Some(processors(busy));
            sample
        };
        let mut reports = Reports::default();
        let first = reports.report(&full(snapshot(1, vec![]), at(0, 0)));
        let mut second_sample = at(0, 0);
        second_sample.machine.processors = Some(Arc::from([
            api::MachineProcessor { kernel_time: 2_000, idle_time: 1_250, ..api::MachineProcessor::default() },
            api::MachineProcessor { kernel_time: 2_000, idle_time: 2_000, ..api::MachineProcessor::default() },
        ]));
        let second = reports.report(&same(snapshot(1, vec![]), second_sample));

        assert_eq!(first.machine.processors.len(), 2);
        assert_eq!(first.machine.processors[0].busy_percent, 0.0);
        assert_eq!(second.machine.processors[0].busy_percent, 75.0);
        assert_eq!(second.machine.processors[1].busy_percent, 0.0);
    }

    #[test]
    fn a_state_follows_its_process_and_changes_when_told() {
        let state = |suspended: bool| api::ProcessState {
            pid: 7,
            sequence_number: sequence(7),
            suspended: Some(suspended),
            ..api::ProcessState::default()
        };
        let listed = |etag: u64, suspended: bool| {
            let mut list = snapshot(etag, vec![info(7, "a.exe", &[])]);
            list.states.value.states = Arc::from([state(suspended)]);
            list
        };
        let mut reports = Reports::default();
        let first = reports.report(&full(listed(1, false), sample(&[row(7, 0)], 0, 0)));
        let unchanged = reports.report(&same(listed(1, true), sample(&[row(7, 0)], 0, 0)));
        let changed = reports.report(&update(
            listed(1, true),
            sample(&[row(7, 0)], 0, 0),
            api::Changes { states: vec![key(7)], ..api::Changes::default() },
        ));

        assert_eq!(first.processes[0].state.suspended, Some(false));
        assert_eq!(unchanged.processes[0].state.suspended, Some(false), "nothing said it changed");
        assert_eq!(changed.processes[0].state.suspended, Some(true));
    }

    #[test]
    fn a_process_seen_for_the_first_time_has_no_cpu_yet() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 10_000, 0)));
        let report = reports.report(&update(
            snapshot(2, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), row(8, 900_000)], 11_000, 0),
            api::Changes { passports: vec![key(8)], states: vec![key(8)], ..api::Changes::default() },
        ));

        let by_pid = |pid: u32| report.processes.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(by_pid(8).cpu_percent, 0.0);
    }
}
