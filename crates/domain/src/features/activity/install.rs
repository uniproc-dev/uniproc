use std::time::Duration;

use app_contracts::features::activity::{ActivityState, Clock};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
use guinea::prelude::*;

use super::actor::{ActivityActor, Flush, Refresh};
use super::clock;

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
    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        let deps = app.try_require::<ActivityDeps>().map_or_else(ActivityDeps::default, |deps| *deps);
        let (_, addr) = app
            .state::<ActivityState>()
            .driven_by(move |push| ActivityActor::new(push, deps));
        app.export::<ActivityState>()?;
        addr.subscribe_on::<WindowsProcessEvents>(Bus::Global);
        addr.subscribe_on::<WindowsReportMessage>(Bus::Global);
        app.every(Pace::Refresh, &addr, || Refresh).named("activity-refresh");
        app.every(Pace::Flush, &addr, || Flush).named("activity-flush");
        addr.send(Refresh);
        Ok(())
    }
}
