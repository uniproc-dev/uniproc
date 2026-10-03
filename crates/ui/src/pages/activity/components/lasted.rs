use crate::l10n::L10n;

struct Ticks;

#[expect(non_upper_case_globals)]
impl Ticks {
    const Second: u64 = 10_000_000;
    const Tenth: u64 = Self::Second / 10;
}

pub fn lasted(l10n: &L10n, ticks: u64) -> String {
    let seconds = (ticks / Ticks::Second) as i64;
    if seconds < 10 {
        l10n.activity_seconds((ticks / Ticks::Tenth) as f64 / 10.0)
    } else if seconds < 60 {
        l10n.activity_seconds(seconds)
    } else if seconds < 3600 {
        l10n.activity_minutes(seconds / 60, seconds % 60)
    } else {
        l10n.activity_hours(seconds / 3600, seconds / 60 % 60)
    }
}
