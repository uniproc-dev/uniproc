use app_contracts::features::activity::Clock;

pub struct Ticks;

#[expect(non_upper_case_globals)]
impl Ticks {
    pub const Second: u64 = 10_000_000;
}

struct Seconds;

#[expect(non_upper_case_globals)]
impl Seconds {
    const Minute: u64 = 60;
    const Day: u64 = 24 * 60 * Self::Minute;
}

struct Marks;

#[expect(non_upper_case_globals)]
impl Marks {
    const Steps: [u64; 11] = [1, 2, 5, 10, 15, 30, 60, 120, 180, 360, 720];
    const Most: u64 = 6;
}

fn of_day(clock: Clock) -> u64 {
    (u64::from(clock.hour) * 60 + u64::from(clock.minute)) * Seconds::Minute + u64::from(clock.second)
}

pub fn shifted(clock: Clock, seconds: i64) -> Clock {
    let day = Seconds::Day as i64;
    let at = (of_day(clock) as i64 + seconds).rem_euclid(day) as u64;
    Clock {
        hour: (at / 3600) as u8,
        minute: (at / 60 % 60) as u8,
        second: (at % 60) as u8,
    }
}

pub fn ticks(now: u64, now_clock: Clock, length: u64) -> Vec<(u64, Clock)> {
    let seconds = length / Ticks::Second;
    let step = Marks::Steps
        .iter()
        .map(|minutes| minutes * Seconds::Minute)
        .find(|step| step * Marks::Most >= seconds)
        .unwrap_or(Seconds::Day);
    let since = of_day(now_clock) % step;
    let start = now.saturating_sub(length);
    let mut found: Vec<(u64, Clock)> = (0..)
        .map(|n: u64| since + n * step)
        .map(|back| (now.saturating_sub(back * Ticks::Second), shifted(now_clock, -(back as i64))))
        .take_while(|(at, _)| *at >= start && *at > 0)
        .collect();
    found.reverse();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: u64 = 3600 * Ticks::Second;

    fn clock(hour: u8, minute: u8, second: u8) -> Clock {
        Clock { hour, minute, second }
    }

    #[test]
    fn the_clock_wraps_round_midnight_both_ways() {
        assert_eq!(shifted(clock(0, 0, 10), -20), clock(23, 59, 50));
        assert_eq!(shifted(clock(23, 59, 50), 20), clock(0, 0, 10));
    }

    #[test]
    fn ticks_fall_on_round_local_minutes_inside_the_window() {
        let now = 1000 * HOUR;

        let labels: Vec<Clock> = ticks(now, clock(1, 34, 20), HOUR).into_iter().map(|(_, at)| at).collect();

        assert_eq!(
            labels,
            [clock(0, 40, 0), clock(0, 50, 0), clock(1, 0, 0), clock(1, 10, 0), clock(1, 20, 0), clock(1, 30, 0)]
        );
    }
}
