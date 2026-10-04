use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityMsg, ActivityState, ApplyPreset, Area, ClearArea, DeletePreset, Filter, Hide, Hover, NewOnly, Only,
    PickArea, Preset, SavePreset, Search, ShowCame, ShowSeries, ShowSpan, ShowWent, Span, Unhide, UnhideInPreset,
};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
use guinea::prelude::*;

use super::install::ActivityDeps;
use super::log::{row, view, Ask, Log};
use super::settings::{remember, remember_presets, ActivitySettings};

pub struct ActivityActor {
    push: Push<ActivityState>,
    deps: ActivityDeps,
    settings: ActivitySettings,
    log: Log,
    span: Span,
    filter: Filter,
    presets: Vec<Preset>,
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
    pub fn new(push: Push<ActivityState>, deps: ActivityDeps, settings: ActivitySettings, seed: &ActivityState) -> Self {
        Self {
            push,
            deps,
            settings,
            log: Log::default(),
            span: seed.span,
            filter: seed.filter.clone(),
            presets: seed.presets.clone(),
            area: None,
            stale: false,
        }
    }

    fn remember(&self) {
        if let Err(err) = remember(&self.settings, self.span, &self.filter) {
            tracing::warn!(?err, "could not remember the activity choices");
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

    fn represet(&mut self, change: impl FnOnce(&mut Vec<Preset>)) {
        change(&mut self.presets);
        if let Err(err) = remember_presets(&self.settings, &self.presets) {
            tracing::warn!(?err, "could not keep the activity presets");
        }
        self.push.send(ActivityMsg::Presets(self.presets.clone()));
    }

    fn refilter(&mut self, change: impl FnOnce(&mut Filter)) {
        change(&mut self.filter);
        self.remember();
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
            ShowSeries, Only, Hide, Unhide, Search, PickArea, ClearArea, Hover, SavePreset, ApplyPreset, DeletePreset,
            UnhideInPreset
        }
    }
}

#[handler]
fn save_preset(this: &mut ActivityActor, SavePreset(name): SavePreset) {
    let filter = Filter {
        text: String::new(),
        ..this.filter.clone()
    };
    this.represet(|presets| match presets.iter_mut().find(|preset| preset.name == name) {
        Some(preset) => preset.filter = filter,
        None => presets.push(Preset { name, filter }),
    });
}

#[handler]
fn apply_preset(this: &mut ActivityActor, ApplyPreset(name): ApplyPreset) {
    let Some(preset) = this.presets.iter().find(|preset| preset.name == name) else {
        return;
    };
    let chosen = preset.filter.clone();
    this.refilter(|filter| {
        *filter = Filter {
            text: std::mem::take(&mut filter.text),
            ..chosen
        }
    });
}

#[handler]
fn delete_preset(this: &mut ActivityActor, DeletePreset(name): DeletePreset) {
    this.represet(|presets| presets.retain(|preset| preset.name != name));
}

#[handler]
fn unhide_in_preset(this: &mut ActivityActor, UnhideInPreset { preset: name, pick }: UnhideInPreset) {
    let in_use = this.presets.iter().any(|preset| preset.name == name && preset.is(&this.filter));
    this.represet(|presets| {
        if let Some(preset) = presets.iter_mut().find(|preset| preset.name == name) {
            preset.filter.hidden.retain(|hidden| *hidden != pick);
        }
    });
    if in_use {
        this.refilter(|filter| filter.hidden.retain(|hidden| *hidden != pick));
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
    this.remember();
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
        shortest: area.shortest.min(area.longest),
        longest: area.shortest.max(area.longest),
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
