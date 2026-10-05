use std::sync::Arc;

use app_contracts::features::activity::{
    ActivityMsg, ActivityRow, ActivityState, ActivityView, AddGroup, Area, ClearArea, DeleteGroup, DropRule, Filter, Group,
    Hide, Hover, Hue, MoveGroup, NewGroup, NewOnly, Only, Pick, PickArea, PutInGroup, RecolorGroup, RenameGroup, Search,
    ShowCame, ShowGroup, ShowOther, ShowSeries, ShowSpan, ShowWent, Span, Unhide,
};
use app_contracts::features::agents::{ProcessInstance, WindowsProcessEvents, WindowsReport, WindowsReportMessage};
use app_contracts::OrWarn;
use guinea::prelude::*;

use super::install::ActivityDeps;
use super::log::{row, view, Ask, Log};
use super::settings::{remember, remember_groups, ActivitySettings};

enum Heard {
    Events(WindowsProcessEvents),
    Running(Arc<WindowsReport>),
}

struct Asked {
    asked: u64,
    now: u64,
    span: Span,
    filter: Filter,
    groups: Vec<Group>,
    area: Option<Area>,
}

pub struct Worked {
    log: Log,
    view: Option<(u64, ActivityView)>,
    hovered: Option<Option<ActivityRow>>,
}

pub struct ActivityActor {
    push: Push<ActivityState>,
    deps: ActivityDeps,
    settings: ActivitySettings,
    log: Option<Log>,
    heard: Vec<Heard>,
    span: Span,
    filter: Filter,
    groups: Vec<Group>,
    area: Option<Area>,
    asked: u64,
    wanted: bool,
    hovering: Option<Option<ProcessInstance>>,
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
            log: Some(Log::default()),
            heard: Vec::new(),
            span: seed.span,
            filter: seed.filter.clone(),
            groups: seed.groups.clone(),
            area: None,
            asked: 0,
            wanted: false,
            hovering: None,
            stale: false,
        }
    }

    fn remember(&self) {
        remember(&self.settings, self.span, &self.filter).or_warn("could not remember the activity choices");
    }

    fn work(&mut self, cx: &Cx<ActivityActor>) {
        let Some(mut log) = self.log.take() else {
            return;
        };
        let heard = std::mem::take(&mut self.heard);
        let asked = self.wanted.then(|| Asked {
            asked: self.asked,
            now: (self.deps.now)(),
            span: self.span,
            filter: self.filter.clone(),
            groups: self.groups.clone(),
            area: self.area,
        });
        self.wanted = false;
        if asked.is_some() {
            self.stale = false;
        }
        let hover = self.hovering.take().map(|key| (key, self.groups.clone()));
        let clock = self.deps.clock;
        cx.spawn_bg(async move {
            for heard in heard {
                match heard {
                    Heard::Events(batch) => log.take(&batch),
                    Heard::Running(report) => log.note_running(&report.processes),
                }
            }
            let view = asked.map(|asked| {
                let ask = Ask {
                    now: asked.now,
                    span: asked.span,
                    filter: &asked.filter,
                    groups: &asked.groups,
                    area: asked.area,
                    clock,
                };
                (asked.asked, view(&log, &ask))
            });
            let hovered = hover.map(|(key, groups)| key.and_then(|key| row(&log, &groups, key, clock)));
            Worked { log, view, hovered }
        });
    }

    fn publish(&mut self, cx: &Cx<ActivityActor>) {
        self.wanted = true;
        self.work(cx);
    }

    fn reask(&mut self, cx: &Cx<ActivityActor>) {
        self.asked += 1;
        self.publish(cx);
    }

    fn regroup(&mut self, change: impl FnOnce(&mut Vec<Group>), cx: &Cx<ActivityActor>) {
        change(&mut self.groups);
        remember_groups(&self.settings, &self.groups).or_warn("could not keep the activity groups");
        self.push.send(ActivityMsg::Groups(self.groups.clone()));
        self.reask(cx);
    }

    fn refilter(&mut self, change: impl FnOnce(&mut Filter), cx: &Cx<ActivityActor>) {
        change(&mut self.filter);
        self.remember();
        self.push.send(ActivityMsg::Filter(self.filter.clone()));
        self.reask(cx);
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
            NewGroup, AddGroup, DropRule, RenameGroup, RecolorGroup, MoveGroup, DeleteGroup, Worked
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
fn show_group(this: &mut ActivityActor, ShowGroup { group, shown }: ShowGroup, cx: Cx) {
    this.regroup(|groups| with_group(groups, &group, |group| group.shown = shown), &cx.detach());
}

#[handler]
fn show_other(this: &mut ActivityActor, ShowOther(shown): ShowOther, cx: Cx) {
    this.refilter(|filter| filter.other = shown, &cx.detach());
}

#[handler]
fn put_in_group(this: &mut ActivityActor, PutInGroup { group, rule }: PutInGroup, cx: Cx) {
    this.regroup(
        |groups| {
            take_rule(groups, &rule);
            with_group(groups, &group, |group| group.rules.push(rule));
        },
        &cx.detach(),
    );
}

#[handler]
fn new_group(this: &mut ActivityActor, NewGroup(rule): NewGroup, cx: Cx) {
    this.regroup(
        |groups| {
            take_rule(groups, &rule);
            let group = Group {
                id: fresh_id(groups),
                name: rule_name(&rule),
                hue: fresh_hue(groups),
                rules: vec![rule],
                shown: true,
            };
            groups.push(group);
        },
        &cx.detach(),
    );
}

#[handler]
fn add_group(this: &mut ActivityActor, AddGroup(name): AddGroup, cx: Cx) {
    let name = name.trim().to_string();
    if name.is_empty() {
        return;
    }
    this.regroup(
        |groups| {
            let group = Group {
                id: fresh_id(groups),
                name,
                hue: fresh_hue(groups),
                rules: Vec::new(),
                shown: true,
            };
            groups.push(group);
        },
        &cx.detach(),
    );
}

#[handler]
fn drop_rule(this: &mut ActivityActor, DropRule { group, rule }: DropRule, cx: Cx) {
    this.regroup(
        |groups| with_group(groups, &group, |group| group.rules.retain(|kept| *kept != rule)),
        &cx.detach(),
    );
}

#[handler]
fn rename_group(this: &mut ActivityActor, RenameGroup { group, name }: RenameGroup, cx: Cx) {
    this.regroup(
        |groups| {
            with_group(groups, &group, |group| {
                if !group.is_built_in() {
                    group.name = name;
                }
            })
        },
        &cx.detach(),
    );
}

#[handler]
fn recolor_group(this: &mut ActivityActor, RecolorGroup { group, hue }: RecolorGroup, cx: Cx) {
    this.regroup(|groups| with_group(groups, &group, |group| group.hue = hue), &cx.detach());
}

#[handler]
fn move_group(this: &mut ActivityActor, MoveGroup { group, up }: MoveGroup, cx: Cx) {
    this.regroup(
        |groups| {
            let Some(at) = groups.iter().position(|kept| kept.id == group) else {
                return;
            };
            let to = if up { at.checked_sub(1) } else { Some(at + 1).filter(|to| *to < groups.len()) };
            if let Some(to) = to {
                groups.swap(at, to);
            }
        },
        &cx.detach(),
    );
}

#[handler]
fn delete_group(this: &mut ActivityActor, DeleteGroup(group): DeleteGroup, cx: Cx) {
    this.regroup(|groups| groups.retain(|kept| kept.id != group || kept.is_built_in()), &cx.detach());
}

#[handler]
fn on_events(this: &mut ActivityActor, batch: WindowsProcessEvents, cx: Cx) {
    this.heard.push(Heard::Events(batch));
    this.stale = true;
    this.work(&cx.detach());
}

#[handler]
fn flush(this: &mut ActivityActor, _msg: Flush, cx: Cx) {
    if this.stale {
        this.publish(&cx.detach());
    }
}

#[handler]
fn on_report(this: &mut ActivityActor, msg: WindowsReportMessage, cx: Cx) {
    if let WindowsReportMessage::Report(report) = msg {
        this.heard.push(Heard::Running(report));
        this.work(&cx.detach());
    }
}

#[handler]
fn refresh(this: &mut ActivityActor, _msg: Refresh, cx: Cx) {
    this.publish(&cx.detach());
}

#[handler]
fn on_worked(this: &mut ActivityActor, Worked { log, view, hovered }: Worked, cx: Cx) {
    this.log = Some(log);
    match view {
        Some((asked, view)) if asked == this.asked => this.push.send(ActivityMsg::View(Arc::new(view))),
        Some(_) => this.wanted = true,
        None => {}
    }
    if let Some(hovered) = hovered {
        this.push.send(ActivityMsg::Hovered(hovered));
    }
    if !this.heard.is_empty() || this.wanted || this.hovering.is_some() {
        this.work(&cx.detach());
    }
}

#[handler]
fn show_span(this: &mut ActivityActor, ShowSpan(span): ShowSpan, cx: Cx) {
    this.span = span;
    this.area = None;
    this.remember();
    this.push.send(ActivityMsg::Span(span));
    this.reask(&cx.detach());
}

#[handler]
fn show_came(this: &mut ActivityActor, ShowCame(shown): ShowCame, cx: Cx) {
    this.refilter(|filter| filter.came = shown, &cx.detach());
}

#[handler]
fn show_went(this: &mut ActivityActor, ShowWent(shown): ShowWent, cx: Cx) {
    this.refilter(|filter| filter.went = shown, &cx.detach());
}

#[handler]
fn new_only(this: &mut ActivityActor, NewOnly(only): NewOnly, cx: Cx) {
    this.refilter(|filter| filter.new_only = only, &cx.detach());
}

#[handler]
fn show_series(this: &mut ActivityActor, ShowSeries(shown): ShowSeries, cx: Cx) {
    this.refilter(|filter| filter.series = shown, &cx.detach());
}

#[handler]
fn only(this: &mut ActivityActor, Only(pick): Only, cx: Cx) {
    this.refilter(|filter| filter.only = pick, &cx.detach());
}

#[handler]
fn hide(this: &mut ActivityActor, Hide(pick): Hide, cx: Cx) {
    this.refilter(
        |filter| {
            if filter.only.as_ref() == Some(&pick) {
                filter.only = None;
            }
            if !filter.hidden.contains(&pick) {
                filter.hidden.push(pick);
            }
        },
        &cx.detach(),
    );
}

#[handler]
fn unhide(this: &mut ActivityActor, Unhide(pick): Unhide, cx: Cx) {
    this.refilter(|filter| filter.hidden.retain(|hidden| *hidden != pick), &cx.detach());
}

#[handler]
fn search(this: &mut ActivityActor, Search(text): Search, cx: Cx) {
    this.refilter(|filter| filter.text = text, &cx.detach());
}

#[handler]
fn pick_area(this: &mut ActivityActor, PickArea(area): PickArea, cx: Cx) {
    this.area = Some(Area {
        from: area.from.min(area.to),
        to: area.from.max(area.to),
        shortest: area.shortest.min(area.longest),
        longest: area.shortest.max(area.longest),
    });
    this.reask(&cx.detach());
}

#[handler]
fn clear_area(this: &mut ActivityActor, _msg: ClearArea, cx: Cx) {
    this.area = None;
    this.reask(&cx.detach());
}

#[handler]
fn hover(this: &mut ActivityActor, Hover(key): Hover, cx: Cx) {
    this.hovering = Some(key);
    this.work(&cx.detach());
}
