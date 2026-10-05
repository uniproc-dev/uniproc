use app_contracts::features::activity::{Area, Lived, Scatter};
use app_contracts::features::agents::ProcessInstance;
use guinea_widgets::chart::scatter::{
    Area as Brushed, Level, Marker, Scale, ScatterEvent, ScatterOptions, ScatterPoint, ScatterSeries,
};
use windows_reactor::Color;

use super::lasted::lasted;
use super::timeline::{ticks, Ticks};
use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, color_f, Palette};

struct Look;

#[expect(non_upper_case_globals)]
impl Look {
    const Size: f32 = 5.0;
    const Faint: u8 = 64;
    const Shortest: f32 = 0.1;
    const Longest: f32 = 3600.0;
    const Lines: [u64; 6] = [
        Ticks::Second / 10,
        Ticks::Second,
        10 * Ticks::Second,
        60 * Ticks::Second,
        600 * Ticks::Second,
        3600 * Ticks::Second,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Hover(Option<ProcessInstance>),
    Pick(Area),
    Clear,
}

fn seconds(ticks: u64) -> f32 {
    (ticks as f64 / Ticks::Second as f64) as f32
}

fn level(lived: Lived) -> Level {
    match lived {
        Lived::Unknown => Level::Below(0),
        Lived::For(ticks) => Level::Value(seconds(ticks)),
        Lived::Running => Level::Above(0),
    }
}

fn lived(level: Level) -> Lived {
    match level {
        Level::Below(_) => Lived::Unknown,
        Level::Value(seconds) => Lived::For((f64::from(seconds) * Ticks::Second as f64).round().max(0.0) as u64),
        Level::Above(_) => Lived::Running,
    }
}

fn marker(lived: Lived) -> Marker {
    match lived {
        Lived::Unknown => Marker::Tick,
        Lived::For(_) => Marker::Dot,
        Lived::Running => Marker::Ring,
    }
}

pub fn series(scatter: &Scatter, palette: Palette) -> Vec<ScatterSeries<ProcessInstance>> {
    let mut series: Vec<ScatterSeries<ProcessInstance>> = Vec::new();
    for dot in scatter.dots() {
        let marker = marker(dot.lived());
        let base = match dot.hue() {
            Some(hue) => palette.hue(hue),
            None if marker == Marker::Tick => palette.critical,
            None => palette.success,
        };
        let color = color_f(if dot.faint() { Color { a: Look::Faint, ..base } } else { base });
        let point = ScatterPoint {
            key: dot.key(),
            at: dot.at(),
            value: level(dot.lived()),
        };
        match series.iter_mut().find(|series| series.marker == marker && series.color == color) {
            Some(series) => series.points.push(point),
            None => series.push(ScatterSeries {
                color,
                marker,
                size: Look::Size,
                points: vec![point],
            }),
        }
    }
    series
}

pub fn options(scatter: &Scatter, l10n: &L10n, palette: Palette) -> ScatterOptions {
    ScatterOptions {
        background: None,
        border: None,
        x: (scatter.now.saturating_sub(scatter.length), scatter.now),
        live: Some(Ticks::Second as f64),
        y: Scale::Log {
            from: Look::Shortest,
            to: Look::Longest,
        },
        above: vec![String::new()],
        below: vec![String::new()],
        y_lines: Look::Lines.iter().map(|&line| (seconds(line), lasted(l10n, line))).collect(),
        x_ticks: ticks(scatter.now, scatter.now_clock, scatter.length)
            .into_iter()
            .map(|(at, clock)| (at, format::clock(clock)))
            .collect(),
        grid: color_f(palette.divider_stroke),
        ink: color_f(palette.secondary_text),
        accent: color_f(accent_color()),
        selection: scatter.area.map(|area| Brushed {
            x: (area.from, area.to),
            y: (level(area.shortest), level(area.longest)),
        }),
        ..ScatterOptions::default()
    }
}

pub fn acts(event: &ScatterEvent<ProcessInstance>, scatter: &Scatter) -> Vec<Act> {
    match event {
        ScatterEvent::Hovered(hit) => {
            vec![Act::Hover(hit.as_ref().map(|hit| hit.key))]
        }
        ScatterEvent::Brushed(area) => vec![Act::Pick(Area {
            from: area.x.0,
            to: area.x.1,
            shortest: lived(area.y.0),
            longest: lived(area.y.1),
        })],
        ScatterEvent::Clicked(_) => scatter.area.map(|_| Act::Clear).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::activity::{Clock, Dot, Hue};
    use guinea_widgets::chart::scatter::{Area as Brushed, Hit, Level, Marker, Scale};

    use super::*;
    use crate::pages::activity::components::timeline::Ticks;

    const HOUR: u64 = 3600 * Ticks::Second;
    const NOW: u64 = 1000 * HOUR;

    fn id(pid: u32) -> ProcessInstance {
        ProcessInstance { pid, sequence: 7 }
    }

    fn dot(pid: u32, lived: Lived, faint: bool) -> Dot {
        Dot::new(id(pid), NOW - HOUR / 2, lived, faint, None)
    }

    fn scatter(dots: Vec<Dot>) -> Scatter {
        Scatter {
            pieces: vec![dots.into()],
            now: NOW,
            now_clock: Clock { hour: 12, minute: 0, second: 0 },
            length: HOUR,
            area: None,
        }
    }

    fn palette() -> Palette {
        Palette::of(windows_reactor::ColorScheme::Dark)
    }

    fn l10n() -> L10n {
        L10n::default()
    }

    #[test]
    fn a_process_is_a_point_by_how_long_it_lived_running_above_and_unknown_below() {
        let scatter = scatter(vec![
            dot(20, Lived::For(2 * Ticks::Second), false),
            dot(21, Lived::Running, false),
            dot(22, Lived::Unknown, false),
        ]);

        let series = series(&scatter, palette());

        let found = |marker: Marker| {
            series
                .iter()
                .filter(|series| series.marker == marker)
                .flat_map(|series| series.points.iter())
                .map(|point| (point.key, point.value))
                .collect::<Vec<_>>()
        };
        assert_eq!(found(Marker::Dot), [(id(20), Level::Value(2.0))], "{series:#?}");
        assert_eq!(found(Marker::Ring), [(id(21), Level::Above(0))], "{series:#?}");
        assert_eq!(found(Marker::Tick), [(id(22), Level::Below(0))], "{series:#?}");
        assert!(series.iter().all(|series| series.points.iter().all(|point| point.at == NOW - HOUR / 2)));
    }

    #[test]
    fn a_faint_process_is_drawn_paler_than_the_rest() {
        let scatter = scatter(vec![
            dot(20, Lived::For(Ticks::Second), false),
            dot(21, Lived::For(Ticks::Second), true),
        ]);

        let series = series(&scatter, palette());

        let alpha = |pid: u32| {
            series
                .iter()
                .find(|series| series.points.iter().any(|point| point.key == id(pid)))
                .map(|series| series.color.a)
        };
        assert!(matches!((alpha(20), alpha(21)), (Some(bold), Some(pale)) if pale < bold), "{series:#?}");
    }

    #[test]
    fn a_process_in_a_group_is_drawn_in_its_hue_whether_its_lifetime_is_known_or_not() {
        let hued = |pid: u32, lived: Lived, hue: Hue| Dot::new(id(pid), NOW - HOUR / 2, lived, false, Some(hue));
        let scatter = scatter(vec![
            hued(20, Lived::For(Ticks::Second), Hue::Purple),
            hued(21, Lived::Unknown, Hue::Purple),
            hued(22, Lived::For(Ticks::Second), Hue::Coral),
            dot(23, Lived::For(Ticks::Second), false),
        ]);

        let series = series(&scatter, palette());

        let color = |pid: u32| {
            series
                .iter()
                .find(|series| series.points.iter().any(|point| point.key == id(pid)))
                .map(|series| series.color)
        };
        let purple = Some(color_f(palette().hue(Hue::Purple)));
        assert_eq!((color(20), color(21)), (purple, purple), "{series:#?}");
        assert_eq!(color(22), Some(color_f(palette().hue(Hue::Coral))), "{series:#?}");
        assert_ne!(color(20), color(22), "{series:#?}");
        assert_ne!(color(20), color(23), "{series:#?}");
    }

    #[test]
    fn the_lifetime_scale_is_labelled_with_durations_and_the_span_moves_with_the_clock() {
        let options = options(&scatter(Vec::new()), &l10n(), palette());

        let labels: Vec<String> =
            options.y_lines.iter().map(|(_, label)| label.replace(['\u{2068}', '\u{2069}'], "")).collect();
        assert!(labels.contains(&"100 ms".to_string()) && labels.contains(&"1 min".to_string()), "{labels:?}");
        assert!(options.above.iter().chain(&options.below).all(String::is_empty), "{options:#?}");
        assert_eq!(options.y, Scale::Log { from: 0.1, to: 3600.0 });
        assert_eq!(options.x, (NOW - HOUR, NOW));
        assert_eq!(options.live, Some(Ticks::Second as f64));
        assert!(!options.x_ticks.is_empty(), "{options:#?}");
    }

    #[test]
    fn a_dragged_rectangle_picks_the_times_and_lifetimes_it_covers() {
        let scatter = scatter(Vec::new());
        let brushed = ScatterEvent::Brushed(Brushed {
            x: (NOW - HOUR / 2, NOW - HOUR / 4),
            y: (Level::Value(0.5), Level::Above(0)),
        });

        assert_eq!(
            acts(&brushed, &scatter),
            [Act::Pick(Area {
                from: NOW - HOUR / 2,
                to: NOW - HOUR / 4,
                shortest: Lived::For(Ticks::Second / 2),
                longest: Lived::Running,
            })]
        );
    }

    #[test]
    fn a_hovered_point_names_its_process_and_a_click_lets_a_picked_area_go() {
        let mut scatter = scatter(vec![dot(20, Lived::Running, false), dot(21, Lived::Unknown, false)]);
        let hit = Hit { series: 0, key: id(21), x: 10.0, y: 10.0 };

        assert_eq!(acts(&ScatterEvent::Hovered(Some(hit)), &scatter), [Act::Hover(Some(id(21)))]);
        assert_eq!(acts(&ScatterEvent::Hovered(None), &scatter), [Act::Hover(None)]);
        assert_eq!(acts(&ScatterEvent::Clicked(None), &scatter), []);

        scatter.area = Some(Area::default());
        assert_eq!(acts(&ScatterEvent::Clicked(None), &scatter), [Act::Clear]);
    }
}
