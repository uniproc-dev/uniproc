use std::time::Duration;

use app_contracts::features::activity::{ActivityState, Clock};
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

use super::actor::{ActivityActor, Flush, Refresh};
use super::clock;
use super::settings::{remembered, remembered_groups, ActivitySettings};

#[derive(Clone, Copy)]
pub struct ActivityDeps {
    pub now: fn() -> u64,
    pub clock: fn(u64) -> Clock,
}

impl Default for ActivityDeps {
    fn default() -> Self {
        Self {
            now: clock::now,
            clock: clock::local,
        }
    }
}

struct Pace;

#[expect(non_upper_case_globals)]
impl Pace {
    const Refresh: Duration = Duration::from_secs(5);
    const Flush: Duration = Duration::from_millis(250);
}

feature! { pub ActivityFeature { exports { ActivityState } } }

#[installs]
fn activity(app: &mut FeatureBuilder, store: amethystate::Store, deps: ActivityDeps) -> anyhow::Result<ActivityFeature> {
    let settings = store.try_settings::<ActivitySettings>()?;
    let (span, filter) = remembered(&settings);
    let seed = ActivityState {
        span,
        filter,
        groups: remembered_groups(&settings),
        ..ActivityState::default()
    };
    let (activity, _) = app
        .state::<ActivityState>()
        .seed(seed.clone())
        .driven_by(move |push| ActivityActor::new(push, deps, settings, &seed));
    Ok(ActivityFeature(activity))
}

pub fn watch(cx: &Install) {
    let clock = cx.spawn(ActivityClock);
    clock.send(Refresh);
    cx.every(Pace::Refresh, || Refresh, clock.clone())
        .named("activity-refresh");
    cx.every(Pace::Flush, || Flush, clock.clone())
        .named("activity-flush");
}

#[derive(Debug)]
struct ActivityClock;

actor! {
    ActivityClock {
        handlers { Refresh, Flush }
        publishes { Refresh, Flush }
    }
}

#[handler]
fn refresh(In(_, _, cx): In<ActivityClock, Refresh>) {
    cx.publish(Refresh);
}

#[handler]
fn flush(In(_, _, cx): In<ActivityClock, Flush>) {
    cx.publish(Flush);
}
