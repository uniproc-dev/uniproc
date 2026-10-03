use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityMsg, ActivityState, Area, ClearArea, Filter, Hide, Hover, NewOnly, Only, PickArea, Search, ShowCame,
    ShowSeries, ShowSpan, ShowWent, Span, Unhide,
};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
use guinea::prelude::*;

use super::install::ActivityDeps;
use super::log::{row, view, Ask, Log};

pub struct ActivityActor {
    push: Push<ActivityState>,
    deps: ActivityDeps,
    log: Log,
    span: Span,
    filter: Filter,
    area: Option<Area>,
    stale: bool,
}

impl std::fmt::Debug for ActivityActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivityActor")
            .field("span", &self.span)
            .field("filter", &self.filter)
            .field("area", &self.area)
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
            area: None,
            stale: false,
        }
    }

    fn publish(&mut self) {
        self.stale = false;
        let view = view(
            &self.log,
            &Ask {
                now: (self.deps.now)(),
                span: self.span,
                filter: &self.filter,
                area: self.area,
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

pub struct Flush;

actor! {
    ActivityActor {
        handlers {
            WindowsProcessEvents, WindowsReportMessage, Refresh, Flush, ShowSpan, ShowCame, ShowWent, NewOnly,
            ShowSeries, Only, Hide, Unhide, Search, PickArea, ClearArea, Hover
        }
    }
}

#[handler]
fn on_events(this: &mut ActivityActor, batch: WindowsProcessEvents) {
    this.log.take(&batch);
    this.stale = true;
}

#[handler]
fn flush(this: &mut ActivityActor, _msg: Flush) {
    if this.stale {
        this.publish();
    }
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
    this.area = None;
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
fn show_series(this: &mut ActivityActor, ShowSeries(shown): ShowSeries) {
    this.refilter(|filter| filter.series = shown);
}

#[handler]
fn only(this: &mut ActivityActor, Only(pick): Only) {
    this.refilter(|filter| filter.only = pick);
}

#[handler]
fn hide(this: &mut ActivityActor, Hide(pick): Hide) {
    this.refilter(|filter| {
        if filter.only.as_ref() == Some(&pick) {
            filter.only = None;
        }
        if !filter.hidden.contains(&pick) {
            filter.hidden.push(pick);
        }
    });
}

#[handler]
fn unhide(this: &mut ActivityActor, Unhide(pick): Unhide) {
    this.refilter(|filter| filter.hidden.retain(|hidden| *hidden != pick));
}

#[handler]
fn search(this: &mut ActivityActor, Search(text): Search) {
    this.refilter(|filter| filter.text = text);
}

#[handler]
fn pick_area(this: &mut ActivityActor, PickArea(area): PickArea) {
    this.area = Some(Area {
        from: area.from.min(area.to),
        to: area.from.max(area.to),
        bottom: area.bottom.min(area.top),
        top: area.bottom.max(area.top),
    });
    this.publish();
}

#[handler]
fn clear_area(this: &mut ActivityActor, _msg: ClearArea) {
    this.area = None;
    this.publish();
}

#[handler]
fn hover(this: &mut ActivityActor, Hover(key): Hover) {
    let hovered = key.and_then(|key| row(&this.log, key, this.deps.clock));
    this.push.send(ActivityMsg::Hovered(hovered));
}
