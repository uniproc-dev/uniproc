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
    const Steps: [u64; 18] = [
        5, 10, 15, 30, 60, 120, 180, 300, 600, 900, 1200, 1800, 3600, 7200, 10800, 14400, 21600, 43200,
    ];
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
        .into_iter()
        .find(|step| step * Marks::Most >= seconds)
        .unwrap_or(Seconds::Day) as i64;
    let since = (of_day(now_clock) as i64) % step;
    let start = now.saturating_sub(length) as i64;
    let second = (now - now % Ticks::Second) as i64;
    let mut found: Vec<(u64, Clock)> = (-1..)
        .map(|n: i64| since + n * step)
        .map(|back| (second - back * Ticks::Second as i64, back))
        .take_while(|(at, _)| *at >= start && *at > 0)
        .map(|(at, back)| (at as u64, shifted(now_clock, -back)))
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
            [
                clock(0, 40, 0),
                clock(0, 50, 0),
                clock(1, 0, 0),
                clock(1, 10, 0),
                clock(1, 20, 0),
                clock(1, 30, 0),
                clock(1, 40, 0)
            ]
        );
    }

    #[test]
    fn a_tick_stays_on_its_second_while_that_second_goes_by() {
        let second = 1000 * HOUR;
        let at = |now: u64| -> Vec<u64> { ticks(now, clock(1, 34, 20), HOUR).into_iter().map(|(at, _)| at).collect() };

        assert_eq!(at(second + 7 * Ticks::Second / 10), at(second));
    }

    #[test]
    fn every_span_is_marked_five_to_seven_times_whatever_the_time() {
        let now = 1000 * HOUR;
        for length in [30, 5 * 60, 15 * 60, 30 * 60, 3600, 24 * 3600].map(|seconds| seconds * Ticks::Second) {
            for minute in 0..60 {
                for second in [0, 7, 30, 59] {
                    let inside = ticks(now, clock(13, minute, second), length)
                        .into_iter()
                        .filter(|(at, _)| (now - length..=now).contains(at))
                        .count();

                    assert!(
                        (5..=7).contains(&inside),
                        "{inside} marks over {} s at 13:{minute}:{second}",
                        length / Ticks::Second
                    );
                }
            }
        }
    }

    #[test]
    fn the_next_mark_is_there_before_the_moving_span_reaches_it() {
        let now = 1000 * HOUR;

        let ahead = ticks(now, clock(13, 37, 0), HOUR).into_iter().filter(|(at, _)| *at > now).count();

        assert_eq!(ahead, 1);
    }
}
