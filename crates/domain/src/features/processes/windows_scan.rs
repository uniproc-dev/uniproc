use std::collections::HashMap;
use std::sync::Arc;

#[cfg(windows)]
mod taskbar {
    use std::ffi::c_void;

    use windows::Win32::{
        DWMWA_CLOAKED, DwmGetWindowAttribute, EnumChildWindows, EnumWindows, GW_OWNER, GWL_EXSTYLE,
        GetClassNameW, GetWindow, GetWindowLongW, GetWindowTextW, GetWindowThreadProcessId, HWND,
        IsWindowVisible, LPARAM, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };
    use windows::core::{BOOL, PWSTR};

    const UWP_FRAME: &str = "ApplicationFrameWindow";
    const UWP_CORE: &str = "Windows.UI.Core.CoreWindow";

    fn class_of(hwnd: HWND) -> String {
        let mut buffer = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, PWSTR(buffer.as_mut_ptr()), buffer.len() as i32) }
            .max(0) as usize;
        String::from_utf16_lossy(&buffer[..len])
    }

    fn title_of(hwnd: HWND) -> String {
        let mut buffer = [0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, PWSTR(buffer.as_mut_ptr()), buffer.len() as i32) }
            .max(0) as usize;
        String::from_utf16_lossy(&buffer[..len])
    }

    fn pid_of(hwnd: HWND) -> u32 {
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        pid
    }

    fn is_cloaked(hwnd: HWND) -> bool {
        let mut cloaked = 0u32;
        let read = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED as u32,
                &mut cloaked as *mut u32 as *mut c_void,
                size_of::<u32>() as u32,
            )
        };
        read.is_ok() && cloaked != 0
    }

    fn is_on_taskbar(hwnd: HWND) -> bool {
        if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return false;
        }
        let owned = !unsafe { GetWindow(hwnd, GW_OWNER as u32) }.0.is_null();
        if owned {
            return false;
        }
        let extended = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
        if extended & WS_EX_TOOLWINDOW != 0 && extended & WS_EX_APPWINDOW == 0 {
            return false;
        }
        !is_cloaked(hwnd)
    }

    struct CoreSearch {
        title: Option<String>,
        pid: Option<u32>,
    }

    unsafe extern "system" fn find_core(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut CoreSearch) };
        let matches = class_of(hwnd) == UWP_CORE
            && search.title.as_ref().is_none_or(|title| title_of(hwnd) == *title);
        if matches {
            search.pid = Some(pid_of(hwnd));
            BOOL(0)
        } else {
            BOOL(1)
        }
    }

    fn uwp_app_pid(frame: HWND) -> Option<u32> {
        let mut child = CoreSearch {
            title: None,
            pid: None,
        };
        unsafe {
            let _ = EnumChildWindows(
                Some(frame),
                Some(find_core),
                LPARAM(&mut child as *mut CoreSearch as isize),
            );
        }
        if child.pid.is_some() {
            return child.pid;
        }

        let title = title_of(frame);
        if title.is_empty() {
            return None;
        }
        let mut detached = CoreSearch {
            title: Some(title),
            pid: None,
        };
        unsafe {
            let _ = EnumWindows(
                Some(find_core),
                LPARAM(&mut detached as *mut CoreSearch as isize),
            );
        }
        detached.pid
    }

    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = unsafe { &mut *(lparam.0 as *mut AppWindows) };
        if is_on_taskbar(hwnd) {
            let pid = if class_of(hwnd) == UWP_FRAME {
                uwp_app_pid(hwnd)
            } else {
                Some(pid_of(hwnd))
            };
            if let Some(pid) = pid.filter(|pid| *pid != 0) {
                windows.add(pid, title_of(hwnd));
            }
        }
        BOOL(1)
    }

    pub fn app_windows() -> AppWindows {
        let mut windows = AppWindows::default();
        unsafe {
            let _ = EnumWindows(Some(collect), LPARAM(&mut windows as *mut AppWindows as isize));
        }
        windows
    }

    use super::AppWindows;
}

#[derive(Debug, Default)]
pub struct AppWindows {
    titles: HashMap<u32, Vec<Arc<str>>>,
}

impl AppWindows {
    pub fn add(&mut self, pid: u32, title: String) {
        self.titles.entry(pid).or_default().push(Arc::from(title));
    }

    pub fn contains(&self, pid: u32) -> bool {
        self.titles.contains_key(&pid)
    }

    pub fn titles(&self, pid: u32) -> &[Arc<str>] {
        self.titles.get(&pid).map_or(&[], Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.titles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.titles.is_empty()
    }
}

#[cfg(windows)]
pub fn app_windows() -> AppWindows {
    taskbar::app_windows()
}

#[cfg(not(windows))]
pub fn app_windows() -> AppWindows {
    AppWindows::default()
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn the_session_has_app_windows() {
        let windows = app_windows();

        assert!(
            !windows.is_empty(),
            "a desktop session always has at least one taskbar window; \
             an empty set means the caller is not in a windowed session"
        );
        assert!(!windows.contains(0));
    }

    #[test]
    fn every_window_of_a_process_is_kept_under_its_pid() {
        let mut windows = AppWindows::default();
        windows.add(7, "First".into());
        windows.add(7, "Second".into());
        windows.add(9, "Other".into());

        let titles: Vec<&str> = windows.titles(7).iter().map(|t| &**t).collect();
        assert_eq!(titles, ["First", "Second"]);
        assert!(windows.titles(8).is_empty());
        assert_eq!(windows.len(), 2);
    }
}
