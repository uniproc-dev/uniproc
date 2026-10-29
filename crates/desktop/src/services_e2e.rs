use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use app_contracts::features::agents::{
    WindowsAction, WindowsReport, WindowsReportMessage, WindowsServiceStartType,
    WindowsServiceState,
};
use app_contracts::features::services::{Command, Select, ServiceActionKind, ServicesState};
use domain::features::agents::actions::{self, ActsOnWindows, WindowsTransport};
use domain::features::agents::providers::windows::WindowsClient;
use domain::features::services::ServicesFeature;
use futures::future::BoxFuture;
use guinea::app::Harness;
use service_fixture::{Fixture, State};

enum Work {
    Act(WindowsAction, mpsc::SyncSender<Result<u32>>),
    Report(mpsc::SyncSender<Result<WindowsReport>>),
    Quit,
}

struct Live {
    send: mpsc::Sender<Work>,
    worker: Option<JoinHandle<()>>,
}

struct Transport(mpsc::Sender<Work>);

impl ActsOnWindows for Transport {
    fn act(&self, action: WindowsAction) -> BoxFuture<'static, u32> {
        let (send, recv) = mpsc::sync_channel(1);
        self.0
            .send(Work::Act(action, send))
            .expect("live IPC worker is running");
        let result = recv
            .recv_timeout(Duration::from_secs(40))
            .expect("real IPC action completes within 40 seconds")
            .expect("real IPC action succeeds");
        Box::pin(std::future::ready(result))
    }
}

impl Live {
    fn connect() -> Result<Self> {
        let (send, recv) = mpsc::channel();
        let (ready, started) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let run = || -> Result<()> {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()?;
                let client = runtime
                    .block_on(WindowsClient::connect(Duration::from_secs(5), || {
                        Duration::from_millis(500)
                    }))?;
                ready.send(Ok(())).ok();
                while let Ok(work) = recv.recv() {
                    match work {
                        Work::Act(action, send) => {
                            let result = runtime.block_on(async {
                                Ok(tokio::time::timeout(
                                    Duration::from_secs(30),
                                    client.act(action),
                                )
                                .await?)
                            });
                            send.send(result).ok();
                        }
                        Work::Report(send) => {
                            let result = runtime.block_on(async {
                                tokio::time::timeout(Duration::from_secs(15), async {
                                    loop {
                                        if let Some(report) = client.report().await? {
                                            return Ok(report);
                                        }
                                        tokio::time::sleep(Duration::from_millis(25)).await;
                                    }
                                })
                                .await?
                            });
                            send.send(result).ok();
                        }
                        Work::Quit => break,
                    }
                }
                Ok(())
            };
            if let Err(error) = run() {
                ready.send(Err(format!("{error:#}"))).ok();
            }
        });
        started
            .recv_timeout(Duration::from_secs(10))?
            .map_err(anyhow::Error::msg)?;
        Ok(Self {
            send,
            worker: Some(worker),
        })
    }

    fn report(&self) -> Result<WindowsReport> {
        let (send, recv) = mpsc::sync_channel(1);
        self.send.send(Work::Report(send))?;
        recv.recv_timeout(Duration::from_secs(20))?
    }

    fn stop(self) {
        drop(self);
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        self.send.send(Work::Quit).ok();
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                eprintln!("real IPC worker panicked");
            }
        }
    }
}

fn refresh(h: &Harness, live: &Live, name: &str, expected: WindowsServiceState) {
    let until = Instant::now() + Duration::from_secs(25);
    loop {
        let report = live.report().expect("read a real service report");
        h.publish(WindowsReportMessage::Report(Arc::new(report)));
        h.settled();
        if h.state::<ServicesState>()
            .rows()
            .iter()
            .any(|row| row.name.as_ref() == name && row.state == expected)
        {
            return;
        }
        assert!(
            Instant::now() < until,
            "service {name} did not appear as {expected:?} in the real report"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait_state(fixture: &Fixture, state: State) -> u32 {
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        let status = fixture.status().expect("read fixture SCM state");
        if status.current_state == state {
            return status.process_id.unwrap_or(0);
        }
        assert!(
            Instant::now() < until,
            "SCM did not reach {state:?}: {status:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[guinea::test(iterations = 1, exclusive = "real-scm")]
#[ignore = "requires administrator rights, the real Windows monitor and the built service fixture; run xtask services-e2e"]
fn services_scm_e2e(h: &mut Harness) {
    let mut run = || -> Result<()> {
        let executable = std::env::var_os("UNIPROC_SERVICE_FIXTURE_EXE")
            .context("missing service fixture executable")?;
        let name = format!("Uniproc.UiTest.E2E.{}", uuid::Uuid::new_v4());
        let fixture = Fixture::create(std::path::Path::new(&executable), &name)?;
        let live = Live::connect()?;
        crate::test_support::install(h);
        h.app(|app| {
            actions::install(app);
            Ok(())
        })
        .unwrap();
        h.install::<ServicesFeature>(&()).unwrap();
        h.publish(WindowsTransport::Connected(Arc::new(Transport(
            live.send.clone(),
        ))));
        h.settled();
        refresh(h, &live, &name, WindowsServiceState::Stopped);
        {
            let state = h.state::<ServicesState>();
            let row = state
                .rows()
                .iter()
                .find(|row| row.name.as_ref() == name)
                .unwrap();
            ensure!(
                row.startup_type == WindowsServiceStartType::Manual,
                "manual startup mode was not read from SCM"
            );
            ensure!(
                row.account
                    .eq_ignore_ascii_case("NT AUTHORITY\\LocalService"),
                "fixture account: {}",
                row.account
            );
            ensure!(
                row.can_start() && !row.can_stop(),
                "stopped service control availability"
            );
        }
        h.dispatch::<ServicesState>().emit(Select(name.clone()));
        h.settled();
        let mut first_pid = 0;
        for (kind, scm, reported) in [
            (
                ServiceActionKind::Start,
                State::Running,
                WindowsServiceState::Running,
            ),
            (
                ServiceActionKind::Restart,
                State::Running,
                WindowsServiceState::Running,
            ),
            (
                ServiceActionKind::Stop,
                State::Stopped,
                WindowsServiceState::Stopped,
            ),
        ] {
            h.dispatch::<ServicesState>().emit(Command(kind));
            h.settled();
            ensure!(
                h.state::<ServicesState>().failure.is_none(),
                "{kind:?} failed: {:?}",
                h.state::<ServicesState>().failure
            );
            let pid = wait_state(&fixture, scm);
            if kind == ServiceActionKind::Start {
                ensure!(pid != 0, "SCM returned no PID");
                first_pid = pid;
            }
            if kind == ServiceActionKind::Restart {
                ensure!(
                    pid != 0 && pid != first_pid,
                    "restart did not replace the process"
                );
            }
            refresh(h, &live, &name, reported);
            println!("{kind:?}: SCM {scm:?}, PID {pid}, report {reported:?}");
        }
        h.publish(WindowsTransport::Lost);
        h.settled();
        live.stop();
        fixture.remove()?;
        ensure!(
            !service_fixture::exists(&name)?,
            "temporary fixture was not removed"
        );
        println!("Removed temporary fixture {name}");
        Ok(())
    };
    run().unwrap();
}
