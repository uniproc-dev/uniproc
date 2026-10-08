use std::time::Duration;

use futures::future::BoxFuture;
use windows_core::{BOOL, HSTRING, PCWSTR, WIN32_ERROR};

use crate::bindings::{
    CloseHandle, CoInitializeEx, EnumWindows, GetCurrentProcess, GetCurrentProcessId, GetTokenInformation, GetWindow,
    GetWindowThreadProcessId, IsWindowVisible, OpenProcess, OpenProcessToken, PostMessageW, ShellExecuteExW,
    TokenElevation, WaitForSingleObject, COINIT_APARTMENTTHREADED, ERROR_CANCELLED, GW_OWNER, HANDLE, HWND, LPARAM,
    SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, SW_SHOWNORMAL, SYNCHRONIZE, TOKEN_ELEVATION, TOKEN_QUERY, WM_CLOSE, WPARAM,
};

#[derive(Debug)]
pub enum RelaunchError {
    Refused,
    Failed(String),
}

#[derive(Clone, Copy)]
pub struct Elevation {
    pub elevated: fn() -> bool,
    pub asked_at_start: fn() -> bool,
    pub relaunch: fn() -> BoxFuture<'static, Result<(), RelaunchError>>,
    pub close: fn(),
}

impl Default for Elevation {
    fn default() -> Self {
        Self {
            elevated: is_elevated,
            asked_at_start,
            relaunch,
            close: close_own_windows,
        }
    }
}

struct Handover;

#[expect(non_upper_case_globals)]
impl Handover {
    const InProcess: &str = "--in-process";
    const After: &str = "--after";
    const Wait: Duration = Duration::from_secs(10);
}

pub fn wait_for_the_copy_it_replaces() {
    let Some(pid) = std::env::args()
        .skip_while(|arg| arg != Handover::After)
        .nth(1)
        .and_then(|pid| pid.parse::<u32>().ok())
    else {
        return;
    };
    unsafe {
        let process = OpenProcess(SYNCHRONIZE as u32, false, pid);
        if process.0.is_null() {
            return;
        }
        let _ = WaitForSingleObject(process, Handover::Wait.as_millis() as u32);
        let _ = CloseHandle(process);
    }
}

fn asked_at_start() -> bool {
    std::env::args().any(|arg| arg == Handover::InProcess)
}

fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if !OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY as u32, &mut token).as_bool() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0;
        let read = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut core::ffi::c_void),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);
        read.as_bool() && elevation.TokenIsElevated != 0
    }
}

fn relaunch() -> BoxFuture<'static, Result<(), RelaunchError>> {
    let (answer, answered) = futures::channel::oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name("run-as-administrator".into())
        .spawn(move || {
            let _ = answer.send(run_as_administrator());
        });
    Box::pin(async move {
        spawned.map_err(|error| RelaunchError::Failed(error.to_string()))?;
        answered
            .await
            .unwrap_or_else(|_| Err(RelaunchError::Failed("the restart thread ended without an answer".into())))
    })
}

fn run_as_administrator() -> Result<(), RelaunchError> {
    let exe = std::env::current_exe().map_err(|error| RelaunchError::Failed(error.to_string()))?;
    let directory = std::env::current_dir().ok().map(|directory| HSTRING::from(directory.as_path()));
    let verb = HSTRING::from("runas");
    let file = HSTRING::from(exe.as_path());
    let parameters = HSTRING::from(format!(
        "{} {} {}",
        Handover::InProcess,
        Handover::After,
        std::process::id()
    ));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC as u32,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        lpDirectory: directory.as_ref().map_or(PCWSTR::null(), |directory| PCWSTR(directory.as_ptr())),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED as u32);
        if ShellExecuteExW(&mut info).as_bool() {
            return Ok(());
        }
    }
    let error = WIN32_ERROR::from_thread();
    if error.0 == ERROR_CANCELLED as u32 {
        Err(RelaunchError::Refused)
    } else {
        Err(RelaunchError::Failed(windows_core::Error::from(error).message()))
    }
}

fn close_own_windows() {
    unsafe extern "system" fn close_if_ours(hwnd: HWND, _: LPARAM) -> BOOL {
        unsafe {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            let top_level = GetWindow(hwnd, GW_OWNER as u32).0.is_null();
            if pid == GetCurrentProcessId() && top_level && IsWindowVisible(hwnd).as_bool() {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE as u32, WPARAM(0), LPARAM(0));
            }
        }
        true.into()
    }
    unsafe {
        let _ = EnumWindows(Some(close_if_ours), LPARAM(0));
    }
}
