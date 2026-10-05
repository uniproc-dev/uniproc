use std::time::Duration;

use app_contracts::features::activity::{ActivityState, Clock};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
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

pub struct ActivityFeature;

impl AppFeature for ActivityFeature {
    type Exports = (ActivityState,);

    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        let deps = app.require_or_default::<ActivityDeps>();
        let settings = app.settings::<ActivitySettings>()?;
        let (span, filter) = remembered(&settings);
        let seed = ActivityState {
            span,
            filter,
            groups: remembered_groups(&settings),
            ..ActivityState::default()
        };
        let (_, addr) = app
            .state::<ActivityState>()
            .seed(seed.clone())
            .driven_by(move |push| ActivityActor::new(push, deps, settings, &seed));
        addr.subscribe_on::<WindowsProcessEvents>(Bus::Global);
        addr.subscribe_on::<WindowsReportMessage>(Bus::Global);
        addr.subscribe_on::<Refresh>(Bus::Global);
        addr.subscribe_on::<Flush>(Bus::Global);
        Ok(())
    }
}

pub fn watch(cx: &FeatureInitContext) {
    GlobalEventBus::publish(Refresh);
    cx.repeat(Pace::Refresh, || GlobalEventBus::publish(Refresh))
        .named("activity-refresh");
    cx.repeat(Pace::Flush, || GlobalEventBus::publish(Flush))
        .named("activity-flush");
}
