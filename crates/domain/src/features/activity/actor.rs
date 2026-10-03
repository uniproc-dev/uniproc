use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityMsg, ActivityState, ClearRange, Filter, NewOnly, PickBucket, Search, ShowBursts, ShowCame, ShowSpan,
    ShowWent, Span,
};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
use guinea::prelude::*;

use super::install::ActivityDeps;
use super::log::{bucket_range, view, Ask, Log};

pub struct ActivityActor {
    push: Push<ActivityState>,
    deps: ActivityDeps,
    log: Log,
    span: Span,
    filter: Filter,
    range: Option<(u64, u64)>,
}

impl std::fmt::Debug for ActivityActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivityActor")
            .field("span", &self.span)
            .field("filter", &self.filter)
            .field("range", &self.range)
            .finish_non_exhaustive()
    }
}

impl ActivityActor {
    pub fn new(push: Push<ActivityState>, deps: ActivityDeps) -> Self {
        Self {
            push,
            deps,
            log: Log::default(),
            span: Span::default(),
            filter: Filter::default(),
            range: None,
        }
    }

    fn publish(&self) {
        let view = view(
            &self.log,
            &Ask {
                now: (self.deps.now)(),
                span: self.span,
                filter: &self.filter,
                range: self.range,
                clock: self.deps.clock,
            },
        );
        self.push.send(ActivityMsg::View(Rc::new(view)));
    }

    fn refilter(&mut self, change: impl FnOnce(&mut Filter)) {
        change(&mut self.filter);
        self.push.send(ActivityMsg::Filter(self.filter.clone()));
        self.publish();
    }
}

pub struct Refresh;

actor! {
    ActivityActor {
        handlers {
            WindowsProcessEvents, WindowsReportMessage, Refresh, ShowSpan, ShowCame, ShowWent, NewOnly,
            ShowBursts, Search, PickBucket, ClearRange
        }
    }
}

#[handler]
fn on_events(this: &mut ActivityActor, batch: WindowsProcessEvents) {
    this.log.take(&batch);
    this.publish();
}

#[handler]
fn on_report(this: &mut ActivityActor, msg: WindowsReportMessage) {
    if let WindowsReportMessage::Report(report) = msg {
        this.log.note_running(&report.processes);
    }
}

#[handler]
fn refresh(this: &mut ActivityActor, _msg: Refresh) {
    this.publish();
}

#[handler]
fn show_span(this: &mut ActivityActor, ShowSpan(span): ShowSpan) {
    this.span = span;
    this.range = None;
    this.push.send(ActivityMsg::Span(span));
    this.publish();
}

#[handler]
fn show_came(this: &mut ActivityActor, ShowCame(shown): ShowCame) {
    this.refilter(|filter| filter.came = shown);
}

#[handler]
fn show_went(this: &mut ActivityActor, ShowWent(shown): ShowWent) {
    this.refilter(|filter| filter.went = shown);
}

#[handler]
fn new_only(this: &mut ActivityActor, NewOnly(only): NewOnly) {
    this.refilter(|filter| filter.new_only = only);
}

#[handler]
fn show_bursts(this: &mut ActivityActor, ShowBursts(shown): ShowBursts) {
    this.refilter(|filter| filter.bursts = shown);
}

#[handler]
fn search(this: &mut ActivityActor, Search(text): Search) {
    this.refilter(|filter| filter.text = text);
}

#[handler]
fn pick_bucket(this: &mut ActivityActor, PickBucket(index): PickBucket) {
    let picked = bucket_range(&this.log, (this.deps.now)(), this.span, index);
    this.range = if picked == this.range { None } else { picked };
    this.publish();
}

#[handler]
fn clear_range(this: &mut ActivityActor, _msg: ClearRange) {
    this.range = None;
    this.publish();
}
