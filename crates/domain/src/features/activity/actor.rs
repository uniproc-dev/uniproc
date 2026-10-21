use std::sync::Arc;
use std::future::Future;

use app_contracts::features::activity::{
    ActivityMsg, ActivityRow, ActivityState, ActivityView, AddGroup, Area, ClearArea, DeleteGroup, DropRule, Filter, Group,
    Hide, Hover, Hue, MoveGroup, NewGroup, NewOnly, Only, Pick, PickArea, PutInGroup, RecolorGroup, RenameGroup, Search,
    ShowCame, ShowGroup, ShowOther, ShowSeries, ShowSpan, ShowWent, Span, Unhide,
};
use app_contracts::features::agents::{ProcessInstance, WindowsProcessEvents, WindowsReport, WindowsReportMessage};
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

#[derive(Clone, Debug, Event)]
pub struct Refresh;

#[derive(Clone, Debug, Event)]
pub struct Flush;

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
        if let Err(err) = remember(&self.settings, self.span, &self.filter) {
            tracing::warn!(?err, "could not remember the activity choices");
        }
    }

    fn work(&mut self) -> Option<impl Future<Output = Worked> + Send + 'static + use<>> {
        if self.heard.is_empty() && !self.wanted && self.hovering.is_none() {
            return None;
        }
        let mut log = self.log.take()?;
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
        Some(async move {
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
        })
    }

    fn publish(&mut self) {
        self.wanted = true;
    }

    fn reask(&mut self) {
        self.asked += 1;
        self.publish();
    }

    fn regroup(&mut self, change: impl FnOnce(&mut Vec<Group>)) {
        change(&mut self.groups);
        if let Err(err) = remember_groups(&self.settings, &self.groups) {
            tracing::warn!(?err, "could not keep the activity groups");
        }
        self.push.send(ActivityMsg::Groups(self.groups.clone()));
        self.reask();
    }

    fn refilter(&mut self, change: impl FnOnce(&mut Filter)) {
        change(&mut self.filter);
        self.remember();
        self.push.send(ActivityMsg::Filter(self.filter.clone()));
        self.reask();
    }
}

actor! {
    ActivityActor {
        handlers {
            ShowSpan => { task Worked }
            ShowCame => { task Worked }
            ShowWent => { task Worked }
            NewOnly => { task Worked }
            ShowSeries => { task Worked }
            Only => { task Worked }
            Hide => { task Worked }
            Unhide => { task Worked }
            Search => { task Worked }
            PickArea => { task Worked }
            ClearArea => { task Worked }
            Hover => { task Worked }
            ShowGroup => { task Worked }
            ShowOther => { task Worked }
            PutInGroup => { task Worked }
            NewGroup => { task Worked }
            AddGroup => { task Worked }
            DropRule => { task Worked }
            RenameGroup => { task Worked }
            RecolorGroup => { task Worked }
            MoveGroup => { task Worked }
            DeleteGroup => { task Worked }
            Worked => { task Worked loop }
        }
        subscribes {
            WindowsProcessEvents => { task Worked }
            WindowsReportMessage => { task Worked }
            Refresh => { task Worked }
            Flush => { task Worked }
        }
    }
}

#[handler]
fn on_events(In(this, batch, cx): In<ActivityActor, WindowsProcessEvents>) {
    this.heard.push(Heard::Events(batch));
    this.stale = true;
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn on_report(In(this, msg, cx): In<ActivityActor, WindowsReportMessage>) {
    if let WindowsReportMessage::Report(report) = msg {
        this.heard.push(Heard::Running(report));
    }
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn refresh(In(this, _msg, cx): In<ActivityActor, Refresh>) {
    this.publish();
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn flush(In(this, _msg, cx): In<ActivityActor, Flush>) {
    if this.stale {
        this.publish();
    }
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_span(In(this, ShowSpan(span), cx): In<ActivityActor, ShowSpan>) {
    this.span = span;
    this.area = None;
    this.remember();
    this.push.send(ActivityMsg::Span(span));
    this.reask();
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_came(In(this, ShowCame(shown), cx): In<ActivityActor, ShowCame>) {
    this.refilter(|filter| filter.came = shown);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_went(In(this, ShowWent(shown), cx): In<ActivityActor, ShowWent>) {
    this.refilter(|filter| filter.went = shown);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn new_only(In(this, NewOnly(only), cx): In<ActivityActor, NewOnly>) {
    this.refilter(|filter| filter.new_only = only);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_series(In(this, ShowSeries(shown), cx): In<ActivityActor, ShowSeries>) {
    this.refilter(|filter| filter.series = shown);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn only(In(this, Only(pick), cx): In<ActivityActor, Only>) {
    this.refilter(|filter| filter.only = pick);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn hide(In(this, Hide(pick), cx): In<ActivityActor, Hide>) {
    this.refilter(
        |filter| {
            if filter.only.as_ref() == Some(&pick) {
                filter.only = None;
            }
            if !filter.hidden.contains(&pick) {
                filter.hidden.push(pick);
            }
        },
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn unhide(In(this, Unhide(pick), cx): In<ActivityActor, Unhide>) {
    this.refilter(|filter| filter.hidden.retain(|hidden| *hidden != pick));
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn search(In(this, Search(text), cx): In<ActivityActor, Search>) {
    this.refilter(|filter| filter.text = text);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn pick_area(In(this, PickArea(area), cx): In<ActivityActor, PickArea>) {
    this.area = Some(Area {
        from: area.from.min(area.to),
        to: area.from.max(area.to),
        shortest: area.shortest.min(area.longest),
        longest: area.shortest.max(area.longest),
    });
    this.reask();
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn clear_area(In(this, _msg, cx): In<ActivityActor, ClearArea>) {
    this.area = None;
    this.reask();
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn hover(In(this, Hover(key), cx): In<ActivityActor, Hover>) {
    this.hovering = Some(key);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_group(In(this, ShowGroup { group, shown }, cx): In<ActivityActor, ShowGroup>) {
    this.regroup(|groups| with_group(groups, &group, |group| group.shown = shown));
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn show_other(In(this, ShowOther(shown), cx): In<ActivityActor, ShowOther>) {
    this.refilter(|filter| filter.other = shown);
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn put_in_group(In(this, PutInGroup { group, rule }, cx): In<ActivityActor, PutInGroup>) {
    this.regroup(
        |groups| {
            take_rule(groups, &rule);
            with_group(groups, &group, |group| group.rules.push(rule));
        },
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn new_group(In(this, NewGroup(rule), cx): In<ActivityActor, NewGroup>) {
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
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn add_group(In(this, AddGroup(name), cx): In<ActivityActor, AddGroup>) {
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
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn drop_rule(In(this, DropRule { group, rule }, cx): In<ActivityActor, DropRule>) {
    this.regroup(
        |groups| with_group(groups, &group, |group| group.rules.retain(|kept| *kept != rule)),
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn rename_group(In(this, RenameGroup { group, name }, cx): In<ActivityActor, RenameGroup>) {
    if name.trim().is_empty() {
        return;
    }
    this.regroup(
        |groups| {
            with_group(groups, &group, |group| {
                if !group.is_built_in() {
                    group.name = name;
                }
            })
        },
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn recolor_group(In(this, RecolorGroup { group, hue }, cx): In<ActivityActor, RecolorGroup>) {
    this.regroup(|groups| with_group(groups, &group, |group| group.hue = hue));
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn move_group(In(this, MoveGroup { group, up }, cx): In<ActivityActor, MoveGroup>) {
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
    );
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn delete_group(In(this, DeleteGroup(group), cx): In<ActivityActor, DeleteGroup>) {
    this.regroup(|groups| groups.retain(|kept| kept.id != group || kept.is_built_in()));
    if let Some(work) = this.work() {
        cx.task().spawn(work);
    }
}

#[handler]
fn on_worked(In(this, Worked { log, view, hovered }, cx): In<ActivityActor, Worked>) {
    this.log = Some(log);
    match view {
        Some((asked, view)) if asked == this.asked => this.push.send(ActivityMsg::View(Arc::new(view))),
        Some(_) => this.wanted = true,
        None => {}
    }
    if let Some(hovered) = hovered {
        this.push.send(ActivityMsg::Hovered(hovered));
    }
    if let Some(work) = this.work() {
        cx.task().spawn(work);
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
