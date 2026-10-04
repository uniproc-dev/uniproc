use crate::bindings::{
    GCLP_HICON, GCLP_HICONSM, GetClassLongPtrW, HICON, HWND, ICON_BIG, ICON_SMALL, ICON_SMALL2,
    LPARAM, SMTO_ABORTIFHUNG, SMTO_BLOCK, SendMessageTimeoutW, WM_GETICON, WPARAM,
};

use super::bitmap::{hicon_to_rgba, RgbaImage};

const ANSWER_WITHIN_MS: u32 = 100;

fn asked(hwnd: HWND, kind: u32) -> Option<HICON> {
    let mut answer = 0usize;
    let sent = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_GETICON as u32,
            WPARAM(kind as usize),
            LPARAM(0),
            (SMTO_ABORTIFHUNG | SMTO_BLOCK) as u32,
            ANSWER_WITHIN_MS,
            Some(&mut answer),
        )
    };
    (sent.0 != 0 && answer != 0).then(|| HICON(answer as *mut _))
}

fn of_class(hwnd: HWND, index: i32) -> Option<HICON> {
    let icon = unsafe { GetClassLongPtrW(hwnd, index) };
    (icon != 0).then(|| HICON(icon as *mut _))
}

pub fn extract_window_icon_rgba(handle: isize) -> Option<RgbaImage> {
    if handle == 0 {
        return None;
    }
    let hwnd = HWND(handle as *mut _);
    let icon = asked(hwnd, ICON_SMALL2 as u32)
        .or_else(|| asked(hwnd, ICON_SMALL as u32))
        .or_else(|| asked(hwnd, ICON_BIG as u32))
        .or_else(|| of_class(hwnd, GCLP_HICONSM))
        .or_else(|| of_class(hwnd, GCLP_HICON))?;
    unsafe { hicon_to_rgba(icon) }
}
