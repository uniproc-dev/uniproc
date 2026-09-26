use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Arc;

use anyhow::{bail, Result};
use app_contracts::features::processes::WindowCommand;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::{
    IsIconic, PostMessageW, SHObjectProperties, SetForegroundWindow, ShellExecuteW, ShowWindow,
    HWND, LPARAM, SHOP_FILEPATH, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOWNORMAL, WM_CLOSE,
    WPARAM,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellRequest {
    RevealFile(Arc<str>),
    FileProperties(Arc<str>),
    SearchOnline(Arc<str>),
    Window { handle: isize, command: WindowCommand },
}

pub fn run(request: ShellRequest) {
    let result = match &request {
        ShellRequest::RevealFile(path) => reveal_file(path),
        ShellRequest::FileProperties(path) => file_properties(path),
        ShellRequest::SearchOnline(query) => search_online(query),
        ShellRequest::Window { handle, command } => window(*handle, *command),
    };
    if let Err(err) = result {
        tracing::warn!(?request, %err, "shell request failed");
    }
}

fn reveal_file(path: &str) -> Result<()> {
    Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{path}\""))
        .spawn()?;
    Ok(())
}

fn file_properties(path: &str) -> Result<()> {
    let shown = unsafe {
        SHObjectProperties(None, SHOP_FILEPATH as u32, &HSTRING::from(path), PCWSTR::null())
    };
    if !shown.as_bool() {
        bail!("SHObjectProperties refused {path}");
    }
    Ok(())
}

fn search_online(query: &str) -> Result<()> {
    let url = format!("https://www.bing.com/search?q={}", encode_query(query));
    let instance = unsafe {
        ShellExecuteW(
            None,
            &HSTRING::from("open"),
            &HSTRING::from(url.as_str()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if (instance.0 as isize) <= 32 {
        bail!("ShellExecuteW failed with {}", instance.0 as isize);
    }
    Ok(())
}

fn encode_query(query: &str) -> String {
    query
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn window(handle: isize, command: WindowCommand) -> Result<()> {
    let hwnd = HWND(handle as *mut core::ffi::c_void);
    unsafe {
        match command {
            WindowCommand::SwitchTo => {
                if IsIconic(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
                if !SetForegroundWindow(hwnd).as_bool() {
                    bail!("SetForegroundWindow refused");
                }
            }
            WindowCommand::Minimize => {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            }
            WindowCommand::Maximize => {
                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
            }
            WindowCommand::Close => {
                if !PostMessageW(Some(hwnd), WM_CLOSE as u32, WPARAM(0), LPARAM(0)).as_bool() {
                    bail!("PostMessageW(WM_CLOSE) failed");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_is_percent_encoded() {
        assert_eq!(encode_query("my app.exe"), "my+app.exe");
        assert_eq!(encode_query("a&b=c"), "a%26b%3Dc");
        assert_eq!(encode_query("проц"), "%D0%BF%D1%80%D0%BE%D1%86");
    }
}
