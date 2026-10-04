use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use app_contracts::features::system::SystemTool;
use windows_core::{HSTRING, PCWSTR};

use crate::bindings::{
    CoInitializeEx, RegGetValueW, ShellExecuteW, COINIT_APARTMENTTHREADED, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, SW_SHOWNORMAL,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub file: String,
    pub args: String,
}

impl Launch {
    fn new(file: impl Into<String>, args: &str) -> Self {
        Self {
            file: file.into(),
            args: args.to_owned(),
        }
    }
}

enum Target {
    Program { file: &'static str, args: &'static str },
    Settings(&'static str),
    Sysinternals { names: &'static [&'static str], page: &'static str },
    EnvironmentEditor,
}

struct Sysinternals;

#[expect(non_upper_case_globals)]
impl Sysinternals {
    const Downloads: &str = "https://learn.microsoft.com/sysinternals/downloads/";
}

struct EnvironmentEditor;

#[expect(non_upper_case_globals)]
impl EnvironmentEditor {
    const PowerToys: &str = "PowerToys.EnvironmentVariables.exe";
    const Windows: &str = "rundll32.exe";
    const WindowsArgs: &str = "sysdm.cpl,EditEnvironmentVariables";
}

fn target(tool: SystemTool) -> Target {
    let program = |file, args| Target::Program { file, args };
    let sysinternals = |names, page| Target::Sysinternals { names, page };
    match tool {
        SystemTool::TaskManager => program("taskmgr.exe", ""),
        SystemTool::ResourceMonitor => program("resmon.exe", ""),
        SystemTool::PerformanceMonitor => program("perfmon.exe", ""),
        SystemTool::ReliabilityMonitor => program("perfmon.exe", "/rel"),
        SystemTool::SystemInformation => program("msinfo32.exe", ""),
        SystemTool::DirectXDiagnostic => program("dxdiag.exe", ""),
        SystemTool::EventViewer => program("eventvwr.msc", ""),
        SystemTool::Services => program("services.msc", ""),
        SystemTool::TaskScheduler => program("taskschd.msc", ""),
        SystemTool::DeviceManager => program("devmgmt.msc", ""),
        SystemTool::DiskManagement => program("diskmgmt.msc", ""),
        SystemTool::ComputerManagement => program("compmgmt.msc", ""),
        SystemTool::RegistryEditor => program("regedit.exe", ""),
        SystemTool::SystemProperties => program("SystemPropertiesAdvanced.exe", ""),
        SystemTool::EnvironmentVariables => Target::EnvironmentEditor,
        SystemTool::StartupApps => Target::Settings("ms-settings:startupapps"),
        SystemTool::InstalledApps => Target::Settings("ms-settings:appsfeatures"),
        SystemTool::Storage => Target::Settings("ms-settings:storagesense"),
        SystemTool::Power => Target::Settings("ms-settings:powersleep"),
        SystemTool::WindowsUpdate => Target::Settings("ms-settings:windowsupdate"),
        SystemTool::About => Target::Settings("ms-settings:about"),
        SystemTool::ProcessExplorer => sysinternals(&["procexp64.exe", "procexp.exe"], "procexp"),
        SystemTool::ProcessMonitor => sysinternals(&["Procmon64.exe", "Procmon.exe"], "procmon"),
        SystemTool::Autoruns => sysinternals(&["Autoruns64.exe", "Autoruns.exe"], "autoruns"),
        SystemTool::TcpView => sysinternals(&["tcpview64.exe", "tcpview.exe"], "tcpview"),
        SystemTool::RamMap => sysinternals(&["RAMMap64.exe", "RAMMap.exe"], "rammap"),
        SystemTool::VmMap => sysinternals(&["vmmap64.exe", "vmmap.exe"], "vmmap"),
    }
}

fn first_found(names: &[&str], locate: fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    names.iter().find_map(|name| locate(name))
}

pub(super) fn resolve(tool: SystemTool, locate: fn(&str) -> Option<PathBuf>) -> Option<Launch> {
    match target(tool) {
        Target::Program { file, args } => Some(Launch::new(file, args)),
        Target::Settings(uri) => Some(Launch::new(uri, "")),
        Target::Sysinternals { names, .. } => {
            first_found(names, locate).map(|path| Launch::new(path.to_string_lossy(), ""))
        }
        Target::EnvironmentEditor => Some(match locate(EnvironmentEditor::PowerToys) {
            Some(path) => Launch::new(path.to_string_lossy(), ""),
            None => Launch::new(EnvironmentEditor::Windows, EnvironmentEditor::WindowsArgs),
        }),
    }
}

pub(super) fn download(tool: SystemTool) -> Option<Launch> {
    match target(tool) {
        Target::Sysinternals { page, .. } => Some(Launch::new(format!("{}{page}", Sysinternals::Downloads), "")),
        _ => None,
    }
}

pub(super) fn missing(locate: fn(&str) -> Option<PathBuf>) -> Vec<SystemTool> {
    SystemTool::ALL
        .into_iter()
        .filter(|tool| match target(*tool) {
            Target::Sysinternals { names, .. } => first_found(names, locate).is_none(),
            _ => false,
        })
        .collect()
}

pub fn locate(name: &str) -> Option<PathBuf> {
    search_dirs().into_iter().map(|dir| dir.join(name)).find(|path| runnable(path))
}

fn runnable(path: &Path) -> bool {
    if path.metadata().is_ok_and(|meta| meta.is_file()) {
        return true;
    }
    path.symlink_metadata().is_ok_and(|meta| !meta.file_type().is_symlink() && !meta.is_dir())
}

fn search_dirs() -> Vec<PathBuf> {
    let known = |var: &str, below: &str| std::env::var_os(var).map(|root| PathBuf::from(root).join(below));
    let mut dirs: Vec<PathBuf> = [
        known("LOCALAPPDATA", r"Microsoft\WindowsApps"),
        known("LOCALAPPDATA", r"Microsoft\WinGet\Links"),
        known("ProgramFiles", r"PowerToys\WinUI3Apps"),
        known("LOCALAPPDATA", r"PowerToys\WinUI3Apps"),
    ]
    .into_iter()
    .flatten()
    .collect();
    for (root, key) in [
        (HKEY_CURRENT_USER, "Environment"),
        (HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"),
    ] {
        if let Some(path) = registry_path(root, key) {
            dirs.extend(path_entries(&path));
        }
    }
    dirs
}

fn path_entries(path: &str) -> impl Iterator<Item = PathBuf> + '_ {
    path.split(';')
        .map(|entry| entry.trim().trim_matches('"'))
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
}

fn registry_path(root: HKEY, key: &str) -> Option<String> {
    let key = HSTRING::from(key);
    let value = HSTRING::from("Path");
    let mut buffer: Vec<u16> = Vec::new();
    for _ in 0..3 {
        let mut size = (buffer.len() * 2) as u32;
        let data = (!buffer.is_empty()).then(|| buffer.as_mut_ptr().cast());
        let status = unsafe { RegGetValueW(root, &key, &value, RRF_RT_REG_SZ as u32, None, data, Some(&mut size)) };
        match status.0 {
            0 if data.is_some() => {
                let len = (size as usize / 2).min(buffer.len());
                let text = &buffer[..len];
                let text = text.split(|unit| *unit == 0).next().unwrap_or(text);
                return Some(String::from_utf16_lossy(text));
            }
            0 | 234 => buffer = vec![0; size as usize / 2 + 1],
            _ => return None,
        }
    }
    None
}

pub fn launch(launch: Launch) {
    let spawned = std::thread::Builder::new().name("system-tool".into()).spawn(move || {
        if let Err(err) = shell_open(&launch) {
            tracing::warn!(?launch, %err, "system tool did not start");
        }
    });
    if let Err(err) = spawned {
        tracing::warn!(%err, "system tool launcher thread did not start");
    }
}

fn shell_open(launch: &Launch) -> Result<()> {
    let args = HSTRING::from(launch.args.as_str());
    let instance = unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED as u32);
        ShellExecuteW(
            None,
            &HSTRING::from("open"),
            &HSTRING::from(launch.file.as_str()),
            if launch.args.is_empty() { PCWSTR::null() } else { PCWSTR(args.as_ptr()) },
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if (instance.0 as isize) <= 32 {
        bail!("ShellExecuteW failed with {}", instance.0 as isize);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nothing(_: &str) -> Option<PathBuf> {
        None
    }

    fn only_procexp(name: &str) -> Option<PathBuf> {
        (name == "procexp.exe").then(|| PathBuf::from(r"C:\Tools\procexp.exe"))
    }

    #[test]
    fn a_sysinternals_tool_is_missing_until_one_of_its_names_is_found() {
        let missing = missing(only_procexp);
        assert!(!missing.contains(&SystemTool::ProcessExplorer));
        assert!(missing.contains(&SystemTool::ProcessMonitor));
        assert!(!missing.contains(&SystemTool::RegistryEditor), "Windows tools are always there");
        assert_eq!(
            resolve(SystemTool::ProcessExplorer, only_procexp),
            Some(Launch::new(r"C:\Tools\procexp.exe", ""))
        );
        assert_eq!(resolve(SystemTool::ProcessMonitor, only_procexp), None);
    }

    #[test]
    fn environment_variables_open_in_powertoys_when_it_is_there() {
        fn powertoys(name: &str) -> Option<PathBuf> {
            (name == EnvironmentEditor::PowerToys).then(|| PathBuf::from(r"C:\PowerToys\PowerToys.EnvironmentVariables.exe"))
        }
        assert_eq!(
            resolve(SystemTool::EnvironmentVariables, powertoys),
            Some(Launch::new(r"C:\PowerToys\PowerToys.EnvironmentVariables.exe", ""))
        );
        assert_eq!(
            resolve(SystemTool::EnvironmentVariables, nothing),
            Some(Launch::new("rundll32.exe", "sysdm.cpl,EditEnvironmentVariables"))
        );
    }

    #[test]
    fn only_sysinternals_tools_have_a_download_page() {
        assert_eq!(
            download(SystemTool::TcpView),
            Some(Launch::new("https://learn.microsoft.com/sysinternals/downloads/tcpview", ""))
        );
        assert_eq!(download(SystemTool::EventViewer), None);
    }

    #[test]
    fn path_entries_skip_blanks_and_quotes() {
        let entries: Vec<_> = path_entries(r#"C:\A; ;"C:\B C";C:\D\"#).collect();
        assert_eq!(entries, [PathBuf::from(r"C:\A"), PathBuf::from(r"C:\B C"), PathBuf::from(r"C:\D\")]);
    }

    #[test]
    fn the_path_is_read_from_the_registry() {
        let machine = registry_path(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
            .expect("the machine PATH");
        assert!(machine.to_ascii_lowercase().contains(r"\system32"), "{machine}");
        assert!(!machine.contains('%'), "expanded: {machine}");
    }
}
