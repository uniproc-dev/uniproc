use app_contracts::features::activity::{Area, Clock};

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Scale {
    pub now: u64,
    pub length: u64,
    pub width: f64,
}

impl Scale {
    fn per_tick(&self) -> f64 {
        self.width / self.length as f64
    }

    pub fn x(&self, at: u64) -> f64 {
        self.width - (self.now as f64 - at as f64) * self.per_tick()
    }

    pub fn at(&self, x: f64) -> u64 {
        (self.now as f64 - (self.width - x) / self.per_tick()).max(0.0) as u64
    }

    pub fn area(&self, from: f64, to: f64, bottom: f32, top: f32) -> Area {
        Area {
            from: self.at(from.min(to)),
            to: self.at(from.max(to)),
            bottom,
            top,
        }
    }
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
    fn a_dot_slides_left_as_time_passes() {
        let at = 1000 * HOUR;
        let before = Scale { now: at + HOUR / 2, length: HOUR, width: 600.0 };
        let after = Scale { now: before.now + 60 * Ticks::Second, ..before };

        assert!((before.x(at) - 300.0).abs() < 0.001, "{}", before.x(at));
        assert!((after.x(at) - 290.0).abs() < 0.001, "{}", after.x(at));
        assert_eq!(before.at(600.0), before.now);
    }

    #[test]
    fn a_dragged_span_is_the_time_it_covers_either_way_round() {
        let scale = Scale { now: 1000 * HOUR, length: HOUR, width: 600.0 };

        let area = scale.area(450.0, 150.0, 0.1, 0.6);

        assert_eq!((area.from, area.to), (scale.now - 3 * HOUR / 4, scale.now - HOUR / 4));
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
