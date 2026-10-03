use std::time::{SystemTime, UNIX_EPOCH};

use app_contracts::features::activity::Clock;
use windows::Win32::{FileTimeToSystemTime, SystemTimeToTzSpecificLocalTime, FILETIME, SYSTEMTIME};

use super::log::Ticks;

struct Epoch;

#[expect(non_upper_case_globals)]
impl Epoch {
    const UnixAfterWindows: u64 = 11_644_473_600 * Ticks::Second;
}

pub fn now() -> u64 {
    let since_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64 / 100)
        .unwrap_or(0);
    since_unix + Epoch::UnixAfterWindows
}

pub fn local(at: u64) -> Clock {
    let file_time = FILETIME {
        dwLowDateTime: at as u32,
        dwHighDateTime: (at >> 32) as u32,
    };
    let mut universal = SYSTEMTIME::default();
    let mut local = SYSTEMTIME::default();
    let converted = unsafe {
        FileTimeToSystemTime(&file_time, &mut universal).as_bool()
            && SystemTimeToTzSpecificLocalTime(None, &universal, &mut local).as_bool()
    };
    if !converted {
        return Clock::default();
    }
    Clock {
        hour: local.wHour as u8,
        minute: local.wMinute as u8,
        second: local.wSecond as u8,
    }
}
