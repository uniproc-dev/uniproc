use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use windows_service::service::{
    Service, ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl,
    ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use windows_service::{define_windows_service, service_dispatcher};

pub const NAME: &str = "Uniproc.UiTest";
pub const DISPLAY_NAME: &str = "Uniproc UI test service";
pub const DESCRIPTION: &str = "Neutral Uniproc test fixture. Manual start; no background work.";

fn deployed_path() -> Result<PathBuf> {
    let root = std::env::var_os("ProgramData").context("ProgramData is not set")?;
    let root = PathBuf::from(root);
    ensure!(root.is_absolute(), "ProgramData must be absolute");
    Ok(root
        .join("Uniproc")
        .join("ServiceFixture")
        .join("uniproc-service-fixture.exe"))
}

fn deploy(executable: &Path) -> Result<PathBuf> {
    let deployed = deployed_path()?;
    let directory = deployed.parent().unwrap();
    std::fs::create_dir_all(directory)?;
    let same = std::fs::read(&deployed)
        .ok()
        .is_some_and(|bytes| std::fs::read(executable).is_ok_and(|source| source == bytes));
    if !same {
        std::fs::copy(executable, &deployed)?;
    }
    for (path, permissions) in [
        (directory, "*S-1-5-19:(OI)(CI)(RX)"),
        (deployed.as_path(), "*S-1-5-19:(RX)"),
    ] {
        let status = Command::new("icacls.exe")
            .arg(path)
            .args(["/grant", permissions])
            .output()?;
        ensure!(
            status.status.success(),
            "could not grant LocalService read/execute on {}: {}",
            path.display(),
            String::from_utf8_lossy(&status.stderr)
        );
    }
    Ok(deployed)
}

pub struct Fixture {
    service: Option<Service>,
    pub name: String,
}

fn manager(access: ServiceManagerAccess) -> Result<ServiceManager> {
    Ok(ServiceManager::local_computer(None::<&str>, access)?)
}

impl Fixture {
    pub fn create(executable: &Path, name: &str) -> Result<Self> {
        ensure!(
            name == NAME || name.starts_with("Uniproc.UiTest.E2E."),
            "unexpected fixture name"
        );
        ensure!(
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
            "invalid fixture name"
        );
        let executable = deploy(executable)?;
        let service = manager(ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE)?
            .create_service(&ServiceInfo {
                name: name.into(), display_name: DISPLAY_NAME.into(), service_type: ServiceType::OWN_PROCESS,
                start_type: ServiceStartType::OnDemand, error_control: ServiceErrorControl::Normal,
                executable_path: executable, launch_arguments: vec!["run".into(), name.into()],
                dependencies: vec![], account_name: Some("NT AUTHORITY\\LocalService".into()), account_password: None,
            }, ServiceAccess::QUERY_STATUS | ServiceAccess::QUERY_CONFIG | ServiceAccess::CHANGE_CONFIG | ServiceAccess::STOP | ServiceAccess::DELETE)
            .context("create the test service (requires administrator rights; never reuses an existing service)")?;
        let fixture = Self {
            service: Some(service),
            name: name.to_owned(),
        };
        fixture
            .service
            .as_ref()
            .unwrap()
            .set_description(DESCRIPTION)?;
        Ok(fixture)
    }

    pub fn status(&self) -> Result<ServiceStatus> {
        Ok(self
            .service
            .as_ref()
            .context("fixture was removed")?
            .query_status()?)
    }

    pub fn keep(mut self) {
        drop(self.service.take());
    }

    pub fn remove(mut self) -> Result<()> {
        self.cleanup()
    }

    fn cleanup(&mut self) -> Result<()> {
        let Some(service) = self.service.take() else {
            return Ok(());
        };
        remove_service(service)?;
        wait_removed(&self.name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!("could not remove fixture {}: {error:#}", self.name);
        }
    }
}

fn remove_service(service: Service) -> Result<()> {
    service.delete()?;
    if service.query_status()?.current_state != ServiceState::Stopped {
        service.stop()?;
        let until = Instant::now() + Duration::from_secs(15);
        while service.query_status()?.current_state != ServiceState::Stopped {
            ensure!(
                Instant::now() < until,
                "test service did not stop during cleanup"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    Ok(())
}

pub fn remove_installed(executable: &Path) -> Result<()> {
    let service = manager(ServiceManagerAccess::CONNECT)?.open_service(
        NAME,
        ServiceAccess::QUERY_CONFIG
            | ServiceAccess::QUERY_STATUS
            | ServiceAccess::STOP
            | ServiceAccess::DELETE,
    )?;
    let config = service.query_config()?;
    let exe = executable.canonicalize()?;
    let actual = config.executable_path.to_string_lossy();
    let plain = format!("{} run {NAME}", exe.display());
    let quoted = format!("\"{}\" run {NAME}", exe.display());
    let deployed = deployed_path()?;
    let deployed_plain = format!("{} run {NAME}", deployed.display());
    let deployed_quoted = format!("\"{}\" run {NAME}", deployed.display());
    ensure!(
        [plain, quoted, deployed_plain, deployed_quoted]
            .iter()
            .any(|expected| actual.eq_ignore_ascii_case(expected)),
        "fixture executable does not match; refusing to remove it"
    );
    remove_service(service)?;
    wait_removed(NAME)
}

pub fn exists(name: &str) -> Result<bool> {
    match manager(ServiceManagerAccess::CONNECT)?.open_service(name, ServiceAccess::QUERY_STATUS) {
        Ok(_) => Ok(true),
        Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
            Ok(false)
        }
        Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1072) => {
            Ok(true)
        }
        Err(error) => Err(error.into()),
    }
}

fn wait_removed(name: &str) -> Result<()> {
    let until = Instant::now() + Duration::from_secs(20);
    while exists(name)? {
        ensure!(Instant::now() < until, "SCM did not finish deleting {name}");
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

define_windows_service!(ffi_service_main, service_main);

pub fn run(name: &str) -> Result<()> {
    Ok(service_dispatcher::start(name, ffi_service_main)?)
}

fn service_main(arguments: Vec<OsString>) {
    if let Some(name) = arguments.first() {
        if let Err(error) = serve(&name.to_string_lossy()) {
            eprintln!("test service failed: {error:#}");
        }
    }
}

fn serve(name: &str) -> Result<()> {
    let (stop, stopped) = mpsc::channel();
    let status = service_control_handler::register(name, move |control| match control {
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        ServiceControl::Stop | ServiceControl::Shutdown => {
            let _ = stop.send(());
            ServiceControlHandlerResult::NoError
        }
        _ => ServiceControlHandlerResult::NotImplemented,
    })?;
    let report = |state, controls, checkpoint, wait_hint| {
        status.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: state,
            controls_accepted: controls,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint,
            wait_hint,
            process_id: None,
        })
    };
    report(
        ServiceState::StartPending,
        ServiceControlAccept::empty(),
        1,
        Duration::from_secs(5),
    )?;
    report(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        0,
        Duration::ZERO,
    )?;
    stopped.recv()?;
    report(
        ServiceState::StopPending,
        ServiceControlAccept::empty(),
        1,
        Duration::from_secs(5),
    )?;
    report(
        ServiceState::Stopped,
        ServiceControlAccept::empty(),
        0,
        Duration::ZERO,
    )?;
    Ok(())
}

pub use windows_service::service::ServiceState as State;
