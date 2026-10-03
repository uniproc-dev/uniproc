use crate::l10n::L10n;

struct Ticks;

#[expect(non_upper_case_globals)]
impl Ticks {
    const Milli: u64 = 10_000;
    const Second: u64 = 1000 * Self::Milli;
    const Minute: u64 = 60 * Self::Second;
    const Hour: u64 = 60 * Self::Minute;
    const Day: u64 = 24 * Self::Hour;
}

pub fn lasted(l10n: &L10n, ticks: u64) -> String {
    let count = |unit: u64| (ticks / unit) as i64;
    if ticks < Ticks::Second {
        l10n.activity_milliseconds(count(Ticks::Milli))
    } else if ticks < Ticks::Minute {
        l10n.activity_seconds(count(Ticks::Second))
    } else if ticks < Ticks::Hour {
        l10n.activity_minutes(count(Ticks::Minute))
    } else if ticks < Ticks::Day {
        l10n.activity_hours(count(Ticks::Hour))
    } else {
        l10n.activity_days(count(Ticks::Day))
    }
}
