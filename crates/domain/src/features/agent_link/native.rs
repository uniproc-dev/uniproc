#[cfg(windows)]
pub fn open_task_manager() {
    use windows::core::{w, PCWSTR};
    use windows::Win32::{ShellExecuteW, SW_SHOWNORMAL};

    std::thread::spawn(|| {
        let started = unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                w!("taskmgr.exe"),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        let code = started.0 as isize;
        if code <= 32 {
            tracing::warn!(code, "Task Manager did not start");
        }
    });
}

#[cfg(not(windows))]
pub fn open_task_manager() {}
