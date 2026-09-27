#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wsl_only = std::env::args().any(|arg| arg == "--wsl-only");
    let probe_wsl = wsl_only || std::env::args().any(|arg| arg == "--wsl");

    if !wsl_only {
        windows::probe().await?;
    }

    if probe_wsl {
        println!();
        wsl::probe().await?;
    } else {
        println!("\n(skipping the WSL agent; pass --wsl to include it)");
    }

    println!("\nall probes passed");
    Ok(())
}

mod windows {
    use anyhow::{Context, bail};
    use app_contracts::features::agents::{
        ProcessPriority, SignatureStatus, WindowsAction, WindowsProcessStats, WindowsReport,
    };
    use domain::features::agents::backend::AgentBackend;
    use domain::features::agents::providers::windows::{
        WindowsBackend, WindowsReply, WindowsRequest, WindowsRpc,
    };
    use domain::features::agents::rpc::{RpcHandle, RpcService};
    use std::time::{Duration, Instant};
    use uniproc_protocol::meta_capnp::ResponseStatus;

    const CONNECT_TIMEOUT_SECS: u64 = 5;

    pub async fn probe() -> anyhow::Result<()> {
        println!("== windows agent ==");

        std::thread::spawn(etag_probe)
            .join()
            .map_err(|_| anyhow::anyhow!("etag probe panicked"))??;

        let started = Instant::now();
        let handle = RpcHandle::<WindowsRpc>::connect(CONNECT_TIMEOUT_SECS)
            .await
            .context("connect failed - is uniproc-windows-agent running (as admin)?")?;
        println!("connect: ok ({} ms)", started.elapsed().as_millis());

        let latency = WindowsBackend::ping(&handle).await.context("ping failed")?;
        println!("ping via AgentBackend: {latency} ms");

        let report = get_report(&handle).await?;
        print_report(&report);

        concurrency_probe(&handle).await?;
        action_probe(&handle).await?;
        teardown_probe(handle).await?;

        Ok(())
    }

    const ETAG_ATTEMPTS: usize = 5;

    fn etag_probe() -> anyhow::Result<()> {
        compio::runtime::Runtime::new()?.block_on(async {
            let session = WindowsRpc::connect(CONNECT_TIMEOUT_SECS).await?;
            let client = session.remote();

            for attempt in 1..=ETAG_ATTEMPTS {
                let first = client.get_services_request().send().promise.await?;
                let etag = first.get()?.get_meta()?.get_etag();
                let mut again = client.get_services_request();
                again.get().init_meta().set_if_none_match(etag);
                let again = again.send().promise.await?;
                if matches!(again.get()?.get_meta()?.get_status(), Ok(ResponseStatus::NotModified)) {
                    println!("etag: getServices NotModified on attempt {attempt} (etag {etag})");
                    break;
                }
                if attempt == ETAG_ATTEMPTS {
                    bail!("getServices never answered NotModified to its own etag");
                }
            }

            for attempt in 1..=ETAG_ATTEMPTS {
                let first = client.get_processes_request().send().promise.await?;
                let etag = first.get()?.get_meta()?.get_etag();
                let mut again = client.get_processes_request();
                again.get().init_meta().set_if_none_match(etag);
                let again = again.send().promise.await?;
                if matches!(again.get()?.get_meta()?.get_status(), Ok(ResponseStatus::NotModified)) {
                    println!("etag: getProcesses NotModified on attempt {attempt} (etag {etag})");
                    return Ok(());
                }
            }
            bail!("getProcesses never answered NotModified to its own etag")
        })
    }

    async fn action_probe(handle: &RpcHandle<WindowsRpc>) -> anyhow::Result<()> {
        let mut child = std::process::Command::new("ping")
            .args(["-n", "60", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .context("spawn a throwaway ping")?;
        let pid = child.id();

        let priority = action(handle, WindowsAction::SetPriority { pid, priority: ProcessPriority::BelowNormal }).await?;
        println!("action: setPriority(BelowNormal) on throwaway pid {pid} -> code {priority}");

        let kill = action(handle, WindowsAction::Kill { pid }).await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let exited = child.try_wait()?.is_some();
        println!("action: kill on throwaway pid {pid} -> code {kill}, exited: {exited}");
        if !exited {
            let _ = child.kill();
            bail!("the agent answered kill with {kill} but the process is still running");
        }

        let missing = action(handle, WindowsAction::Kill { pid }).await?;
        println!("action: kill on the same, now gone pid -> code {missing}");
        Ok(())
    }

    async fn action(handle: &RpcHandle<WindowsRpc>, action: WindowsAction) -> anyhow::Result<u32> {
        match handle.call(WindowsRequest::Action(action)).await? {
            WindowsReply::Code(code) => Ok(code),
            _ => bail!("agent answered an action with the wrong reply"),
        }
    }

    async fn get_report(handle: &RpcHandle<WindowsRpc>) -> anyhow::Result<WindowsReport> {
        let started = Instant::now();
        match handle.call(WindowsRequest::Scan).await? {
            WindowsReply::Report(Some(report)) => {
                println!(
                    "scan: {} processes joined in {} ms",
                    report.processes.len(),
                    started.elapsed().as_millis()
                );
                Ok(report)
            }
            WindowsReply::Report(None) => bail!("the process list kept moving under the metrics"),
            _ => bail!("agent answered a scan with the wrong reply"),
        }
    }

    fn print_report(report: &WindowsReport) {
        let m = &report.machine;
        println!(
            "  machine: cpu {:.1}% @ {} / {} MHz, mem {} / {} MB, net rx {} tx {}",
            m.cpu_percent,
            m.cpu_current_mhz,
            m.cpu_max_mhz,
            m.used_physical_kb / 1024,
            m.total_physical_kb / 1024,
            m.net_rx_bytes,
            m.net_tx_bytes,
        );

        assert!(m.total_physical_kb > 0, "total physical memory decoded as 0");
        assert!(!report.processes.is_empty(), "no processes in the report");

        let enriched: Vec<&WindowsProcessStats> = report
            .processes
            .iter()
            .filter(|p| p.is_service || p.is_kernel_process)
            .collect();

        println!(
            "  enrichment: {} services, {} kernel, {} signed by microsoft",
            report.processes.iter().filter(|p| p.is_service).count(),
            report.processes.iter().filter(|p| p.is_kernel_process).count(),
            report
                .processes
                .iter()
                .filter(|p| p.signature == SignatureStatus::Microsoft)
                .count(),
        );

        for p in enriched.iter().take(6) {
            println!(
                "    pid={:<6} {:<26} svc={} krn={} sig={:?} {}",
                p.pid,
                truncate(&p.name, 26),
                p.is_service,
                p.is_kernel_process,
                p.signature,
                truncate(&p.image_path, 48),
            );
        }
    }

    async fn concurrency_probe(handle: &RpcHandle<WindowsRpc>) -> anyhow::Result<()> {
        let report_handle = handle.clone();
        let report = tokio::spawn(async move { report_handle.call(WindowsRequest::Scan).await });

        let ping_handle = handle.clone();
        let pings = tokio::spawn(async move {
            let mut worst = Duration::ZERO;
            for _ in 0..15 {
                let started = Instant::now();
                if ping_handle.call(WindowsRequest::Ping).await.is_ok() {
                    worst = worst.max(started.elapsed());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            worst
        });

        report.await??;
        let worst = pings.await?;
        println!("concurrency: worst ping during getReport {} ms", worst.as_millis());
        Ok(())
    }

    async fn teardown_probe(handle: RpcHandle<WindowsRpc>) -> anyhow::Result<()> {
        let orphan = handle.clone();
        drop(handle);

        orphan.call(WindowsRequest::Ping).await.context("clone stopped working after a sibling dropped")?;

        drop(orphan);
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("teardown: session closed on last handle drop");
        Ok(())
    }

    fn truncate(s: &str, max: usize) -> String {
        if s.chars().count() <= max {
            s.to_string()
        } else {
            s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
        }
    }
}

mod wsl {
    use anyhow::{Context, bail};
    use app_contracts::features::agents::{LinuxProcessStats, LinuxReport};
    use domain::features::agents::backend::AgentBackend;
    use domain::features::agents::providers::wsl::{WslBackend, WslReply, WslRequest, WslRpc};
    use domain::features::agents::rpc::RpcHandle;
    use std::time::Instant;

    const CONNECT_TIMEOUT_SECS: u64 = 40;

    pub async fn probe() -> anyhow::Result<()> {
        println!("== wsl agent ==");

        let distro = std::env::var("WSL_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
        let agent_path = std::env::var("WSL_AGENT_PATH")
            .unwrap_or_else(|_| "/usr/local/bin/uniproc-agent".to_string());
        println!("launching {agent_path} in {distro}");
        domain::features::agents::providers::wsl::set_launch_config(distro.clone(), agent_path);

        let started = Instant::now();
        let handle = RpcHandle::<WslRpc>::connect(CONNECT_TIMEOUT_SECS)
            .await
            .context("connect failed - is the Linux agent running inside WSL?")?;
        println!("connect: ok ({} ms)", started.elapsed().as_millis());

        let latency = WslBackend::ping(&handle).await.context("ping failed")?;
        println!("ping via AgentBackend: {latency} ms");

        let _ = handle.call(WslRequest::GetReport).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        match handle.call(WslRequest::GetReport).await? {
            WslReply::Report(report) => {
                let m = &report.machine;
                println!(
                    "getReport: {} processes, {} environments, {} docker containers",
                    report.processes.len(),
                    report.environments.len(),
                    report.docker_containers.len(),
                );
                println!(
                    "  machine: mem {} / {} MB, busy {} ns",
                    m.used_kb / 1024,
                    m.total_kb / 1024,
                    m.busy_ns
                );
                assert!(m.total_kb > 0, "total memory decoded as 0");
                assert!(!report.processes.is_empty(), "no processes in the report");

                let mut top: Vec<_> = report.processes.iter().collect();
                top.sort_by_key(|p| std::cmp::Reverse(p.rss_kb));
                let with_rss = report.processes.iter().filter(|p| p.rss_kb > 0).count();
                println!(
                    "  processes with rss>0: {} / {}",
                    with_rss,
                    report.processes.len()
                );
                for p in top.iter().take(10) {
                    println!(
                        "    pid={:<6} {:<24} cpu={:.1}% rss={} kb",
                        p.global_pid, p.name, p.cpu_percent, p.rss_kb
                    );
                }

                let mut busiest: Vec<_> = report.processes.iter().collect();
                busiest.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
                println!("  busiest by cpu:");
                for p in busiest.iter().take(3) {
                    println!(
                        "    pid={:<6} {:<24} cpu={:.1}% rss={} kb",
                        p.global_pid, p.name, p.cpu_percent, p.rss_kb
                    );
                }

                println!("  environments:");
                for e in &report.environments {
                    let procs = report
                        .processes
                        .iter()
                        .filter(|p| p.mnt_ns == e.mnt_ns && p.pid_ns == e.pid_ns)
                        .count();
                    println!(
                        "    {:?}  mnt_ns={} pid_ns={} procs={} name={:?}",
                        e.kind, e.mnt_ns, e.pid_ns, procs, e.name
                    );
                }
                let homeless: Vec<_> = report
                    .processes
                    .iter()
                    .filter(|p| !report.environments.iter().any(|e| e.pid_ns == p.pid_ns))
                    .collect();
                println!("  in no environment: {}", homeless.len());
                for p in homeless.iter().take(15) {
                    println!(
                        "    pid={:<6} local={:<6} mnt_ns={} pid_ns={} {:?}",
                        p.global_pid, p.local_pid, p.mnt_ns, p.pid_ns, p.name
                    );
                }
                let unnamed = report.processes.iter().filter(|p| p.name.is_empty()).count();
                println!("  unnamed processes: {unnamed}");
            }
            _ => bail!("agent answered getReport with the wrong reply"),
        }

        load_probe(&handle, &distro).await
    }

    const LOAD_SECS: u64 = 4;
    const LOAD: &str = r#"
timeout 4 yes > /dev/null &
python3 -c '
import os, time
end = time.time() + 4
with open("/tmp/uniproc-probe", "wb") as f:
    while time.time() < end:
        f.write(b"\0" * 1048576)
        f.flush()
        os.fsync(f.fileno())
' &
curl -sL -o /dev/null --limit-rate 4M --max-time 4 "https://speed.cloudflare.com/__down?bytes=20000000" &
wait
rm -f /tmp/uniproc-probe
"#;

    async fn report(handle: &RpcHandle<WslRpc>) -> anyhow::Result<LinuxReport> {
        match handle.call(WslRequest::GetReport).await? {
            WslReply::Report(report) => Ok(report),
            _ => bail!("agent answered getReport with the wrong reply"),
        }
    }

    fn started<'a>(report: &'a LinuxReport, before: &LinuxReport, name: &str) -> Option<&'a LinuxProcessStats> {
        report.processes.iter().find(|p| {
            p.name.starts_with(name) && !before.processes.iter().any(|old| old.global_pid == p.global_pid)
        })
    }

    fn net(p: &LinuxProcessStats) -> u64 {
        p.tcp_rx_remote_bytes + p.tcp_tx_remote_bytes + p.udp_rx_remote_bytes + p.udp_tx_remote_bytes
    }

    fn verdict(label: &str, seen: bool, detail: String, missing: &mut Vec<String>) {
        println!("  {:<8} {:<34} {detail}", if seen { "ok" } else { "MISSING" }, label);
        if !seen {
            missing.push(label.to_string());
        }
    }

    async fn load_probe(handle: &RpcHandle<WslRpc>, distro: &str) -> anyhow::Result<()> {
        println!("\n== wsl load: {LOAD_SECS}s of yes, fsync'd writes, a 4 MB/s download ==");
        let before = report(handle).await?;
        let began = Instant::now();
        let mut load = std::process::Command::new("wsl.exe")
            .args(["-d", distro, "--", "sh", "-c", LOAD])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("wsl.exe did not start the load")?;

        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let early = report(handle).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let late = report(handle).await?;
        load.wait()?;
        let after = report(handle).await?;
        let elapsed = began.elapsed().as_secs_f64();

        let mut missing = Vec::new();
        let named = |report, name| started(report, &before, name);
        let cpu = named(&late, "yes").map(|p| p.cpu_percent);
        verdict(
            "process cpu (yes)",
            cpu.is_some_and(|cpu| cpu > 1.0),
            format!("cpu_percent={cpu:?}"),
            &mut missing,
        );
        let written = named(&early, "python3")
            .zip(named(&late, "python3"))
            .map(|(a, b)| b.disk_write_bytes.saturating_sub(a.disk_write_bytes));
        verdict(
            "process disk write (python3)",
            written.is_some_and(|bytes| bytes > 0),
            format!("+{written:?} bytes in 1.5 s"),
            &mut missing,
        );
        let fetched = named(&early, "curl").zip(named(&late, "curl")).map(|(a, b)| net(b).saturating_sub(net(a)));
        verdict(
            "process net remote (curl)",
            fetched.is_some_and(|bytes| bytes > 0),
            format!("+{fetched:?} bytes in 1.5 s"),
            &mut missing,
        );

        let mut movers: Vec<(u64, u64, &str)> = late
            .processes
            .iter()
            .filter_map(|b| {
                let a = early.processes.iter().find(|a| a.global_pid == b.global_pid)?;
                let disk = b.disk_write_bytes.saturating_sub(a.disk_write_bytes);
                let net = net(b).saturating_sub(net(a));
                (disk > 0 || net > 0).then_some((disk, net, b.name.as_str()))
            })
            .collect();
        movers.sort_by_key(|(disk, net, _)| std::cmp::Reverse(disk + net));
        println!("  processes whose disk write or remote net grew in those 1.5 s:");
        for (disk, net, name) in movers.iter().take(8) {
            println!("    {name:<20} disk_write +{disk:<12} net +{net}");
        }

        let (m0, m1) = (&before.machine, &after.machine);
        let busy = m1.busy_ns.saturating_sub(m0.busy_ns) as f64;
        let cores = f64::from(m1.cpu_count.max(1));
        verdict(
            "machine cpu (busy_ns)",
            busy > 0.0,
            format!(
                "+{busy} ns over {elapsed:.1} s on {} cores = {:.1}%",
                m1.cpu_count,
                busy / (elapsed * 1e9 * cores) * 100.0
            ),
            &mut missing,
        );
        verdict(
            "machine disk write",
            m1.disk_write_bytes > m0.disk_write_bytes,
            format!("+{} bytes", m1.disk_write_bytes.saturating_sub(m0.disk_write_bytes)),
            &mut missing,
        );
        verdict(
            "machine net remote rx",
            m1.tcp_rx_remote_bytes > m0.tcp_rx_remote_bytes,
            format!("+{} bytes", m1.tcp_rx_remote_bytes.saturating_sub(m0.tcp_rx_remote_bytes)),
            &mut missing,
        );

        if missing.is_empty() {
            Ok(())
        } else {
            bail!("the agent did not show: {}", missing.join(", "))
        }
    }
}
