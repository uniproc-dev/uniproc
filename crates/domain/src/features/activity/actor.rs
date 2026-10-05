use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityMsg, ActivityState, AddGroup, Area, ClearArea, DeleteGroup, DropRule, Filter, Group, Hide, Hover, Hue, MoveGroup,
    NewGroup, NewOnly, Only, Pick, PickArea, PutInGroup, RecolorGroup, RenameGroup, Search, ShowCame, ShowGroup, ShowOther,
    ShowSeries, ShowSpan, ShowWent, Span, Unhide,
};
use app_contracts::features::agents::{WindowsProcessEvents, WindowsReportMessage};
use app_contracts::OrWarn;
use guinea::prelude::*;

use super::install::ActivityDeps;
use super::log::{row, view, Ask, Log};
use super::settings::{remember, remember_groups, ActivitySettings};

pub struct ActivityActor {
    push: Push<ActivityState>,
    deps: ActivityDeps,
    settings: ActivitySettings,
    log: Log,
    span: Span,
    filter: Filter,
    groups: Vec<Group>,
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
            groups: seed.groups.clone(),
            area: None,
            stale: false,
        }
    }

    fn remember(&self) {
        remember(&self.settings, self.span, &self.filter).or_warn("could not remember the activity choices");
    }

    fn publish(&mut self) {
        self.stale = false;
        let view = view(
            &self.log,
            &Ask {
                now: (self.deps.now)(),
                span: self.span,
                filter: &self.filter,
                groups: &self.groups,
                area: self.area,
                clock: self.deps.clock,
            },
        );
        self.push.send(ActivityMsg::View(Rc::new(view)));
    }

    fn regroup(&mut self, change: impl FnOnce(&mut Vec<Group>)) {
        change(&mut self.groups);
        remember_groups(&self.settings, &self.groups).or_warn("could not keep the activity groups");
        self.push.send(ActivityMsg::Groups(self.groups.clone()));
        self.publish();
    }

    fn refilter(&mut self, change: impl FnOnce(&mut Filter)) {
        change(&mut self.filter);
        self.remember();
        self.push.send(ActivityMsg::Filter(self.filter.clone()));
        self.publish();
    }
}

#[derive(Clone, Debug, Event)]
pub struct Refresh;

#[derive(Clone, Debug, Event)]
pub struct Flush;

actor! {
    ActivityActor {
        handlers {
            WindowsProcessEvents, WindowsReportMessage, Refresh, Flush, ShowSpan, ShowCame, ShowWent, NewOnly,
            ShowSeries, Only, Hide, Unhide, Search, PickArea, ClearArea, Hover, ShowGroup, ShowOther, PutInGroup,
            NewGroup, AddGroup, DropRule, RenameGroup, RecolorGroup, MoveGroup, DeleteGroup
        }
    }
}

fn with_group(groups: &mut [Group], id: &str, change: impl FnOnce(&mut Group)) {
    if let Some(group) = groups.iter_mut().find(|group| group.id == id) {
        change(group);
    }
}

fn take_rule(groups: &mut [Group], rule: &Pick) {
    for group in groups {
        group.rules.retain(|kept| kept != rule);
    }
}

fn rule_name(rule: &Pick) -> String {
    match rule {
        Pick::Exe(name) | Pick::Under(name) | Pick::Folder(name) => name.to_string(),
    }
}

fn fresh_id(groups: &[Group]) -> String {
    (1..)
        .map(|n| format!("g{n}"))
        .find(|id| groups.iter().all(|group| group.id != *id))
        .unwrap_or_default()
}

fn fresh_hue(groups: &[Group]) -> Hue {
    Hue::ALL
        .into_iter()
        .find(|hue| groups.iter().all(|group| group.hue != *hue))
        .unwrap_or(Hue::ALL[groups.len() % Hue::ALL.len()])
}

#[handler]
fn show_group(this: &mut ActivityActor, ShowGroup { group, shown }: ShowGroup) {
    this.regroup(|groups| with_group(groups, &group, |group| group.shown = shown));
}

#[handler]
fn show_other(this: &mut ActivityActor, ShowOther(shown): ShowOther) {
    this.refilter(|filter| filter.other = shown);
}

#[handler]
fn put_in_group(this: &mut ActivityActor, PutInGroup { group, rule }: PutInGroup) {
    this.regroup(|groups| {
        take_rule(groups, &rule);
        with_group(groups, &group, |group| group.rules.push(rule));
    });
}

#[handler]
fn new_group(this: &mut ActivityActor, NewGroup(rule): NewGroup) {
    this.regroup(|groups| {
        take_rule(groups, &rule);
        let group = Group {
            id: fresh_id(groups),
            name: rule_name(&rule),
            hue: fresh_hue(groups),
            rules: vec![rule],
            shown: true,
        };
        groups.push(group);
    });
}

#[handler]
fn add_group(this: &mut ActivityActor, AddGroup(name): AddGroup) {
    let name = name.trim().to_string();
    if name.is_empty() {
        return;
    }
    this.regroup(|groups| {
        let group = Group {
            id: fresh_id(groups),
            name,
            hue: fresh_hue(groups),
            rules: Vec::new(),
            shown: true,
        };
        groups.push(group);
    });
}

#[handler]
fn drop_rule(this: &mut ActivityActor, DropRule { group, rule }: DropRule) {
    this.regroup(|groups| with_group(groups, &group, |group| group.rules.retain(|kept| *kept != rule)));
}

#[handler]
fn rename_group(this: &mut ActivityActor, RenameGroup { group, name }: RenameGroup) {
    this.regroup(|groups| {
        with_group(groups, &group, |group| {
            if !group.is_built_in() {
                group.name = name;
            }
        })
    });
}

#[handler]
fn recolor_group(this: &mut ActivityActor, RecolorGroup { group, hue }: RecolorGroup) {
    this.regroup(|groups| with_group(groups, &group, |group| group.hue = hue));
}

#[handler]
fn move_group(this: &mut ActivityActor, MoveGroup { group, up }: MoveGroup) {
    this.regroup(|groups| {
        let Some(at) = groups.iter().position(|kept| kept.id == group) else {
            return;
        };
        let to = if up { at.checked_sub(1) } else { Some(at + 1).filter(|to| *to < groups.len()) };
        if let Some(to) = to {
            groups.swap(at, to);
        }
    });
}

#[handler]
fn delete_group(this: &mut ActivityActor, DeleteGroup(group): DeleteGroup) {
    this.regroup(|groups| groups.retain(|kept| kept.id != group || kept.is_built_in()));
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
    let hovered = key.and_then(|key| row(&this.log, &this.groups, key, this.deps.clock));
    this.push.send(ActivityMsg::Hovered(hovered));
}
