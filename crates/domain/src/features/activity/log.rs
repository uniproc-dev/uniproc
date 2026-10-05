use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use app_contracts::features::activity::{
    ActivityRow, ActivityView, Area, Came, Clock, Dot, Exit, Filter, Group, Hue, Launcher, Legend, Lived, Pick,
    Scatter, Series, Span, Went,
};
use app_contracts::features::agents::{
    ProcessCame, ProcessEvent, ProcessInstance, ProcessWent, WindowsProcessEvents, WindowsProcessStats,
};

pub struct Ticks;

#[expect(non_upper_case_globals)]
impl Ticks {
    pub const Second: u64 = 10_000_000;
    pub const Minute: u64 = 60 * Self::Second;
}

struct Pace;

#[expect(non_upper_case_globals)]
impl Pace {
    const HalfMinute: u64 = 30 * Ticks::Second;
    const FiveMinutes: u64 = 5 * Ticks::Minute;
    const Quarter: u64 = 15 * Ticks::Minute;
    const HalfHour: u64 = 30 * Ticks::Minute;
    const Hour: u64 = 60 * Ticks::Minute;
    const Day: u64 = 24 * Self::Hour;
    const SeriesLeast: usize = 3;
    const Routine: usize = 20;
    const ChainDepth: usize = 16;
    const Shown: usize = 200;
    const Shells: [&str; 5] = ["cmd.exe", "powershell.exe", "pwsh.exe", "wscript.exe", "cscript.exe"];
    const ConsoleHost: &str = "conhost.exe";
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Came,
    Went,
}

struct Started {
    came: ProcessCame,
    name: Arc<str>,
    file: Arc<str>,
    folder: Arc<str>,
    first_seen: bool,
    launched: Option<(ProcessInstance, Arc<str>)>,
}

impl Started {
    fn picks(&self) -> Vec<Pick> {
        let mut picks = Vec::new();
        if !self.file.is_empty() {
            picks.push(Pick::Exe(self.file.clone()));
        }
        if !self.folder.is_empty() {
            picks.push(Pick::Folder(self.folder.clone()));
        }
        if let Some((_, launcher)) = &self.launched {
            picks.push(Pick::Under(launcher.clone()));
        }
        picks
    }
}

struct Known {
    name: Arc<str>,
    start_time: u64,
}

#[derive(Default)]
pub struct Log {
    started: HashMap<ProcessInstance, Started>,
    ended: HashMap<ProcessInstance, ProcessWent>,
    timeline: BTreeSet<(u64, Kind, ProcessInstance)>,
    seen: HashSet<Arc<str>>,
    habits: HashMap<(Arc<str>, Arc<str>), usize>,
    unplaced: Vec<ProcessInstance>,
    running: HashMap<u32, Known>,
    since: Option<u64>,
    lost: u64,
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

fn place(path: &str) -> (Arc<str>, Arc<str>) {
    let exe = path.to_lowercase();
    let folder = exe.rsplit_once(['\\', '/']).map_or("", |(folder, _)| folder);
    (Arc::from(exe.as_str()), Arc::from(folder))
}

fn admitted(filter: &Filter, has: impl Fn(&Pick) -> bool) -> bool {
    filter.only.as_ref().is_none_or(&has) && !filter.hidden.iter().any(has)
}

fn group_of(groups: &[Group], has: impl Fn(&Pick) -> bool) -> Option<usize> {
    groups.iter().position(|group| group.rules.iter().any(&has))
}

fn name_of(came: &ProcessCame) -> Arc<str> {
    if came.image_path.is_empty() {
        came.command_line.split_whitespace().next().map(file_name).unwrap_or_default().into()
    } else {
        file_name(&came.image_path).into()
    }
}

fn is_shell(name: &str) -> bool {
    Pace::Shells.iter().any(|shell| shell.eq_ignore_ascii_case(name))
}

impl Log {
    pub fn record(&mut self, events: &[ProcessEvent]) {
        for event in events {
            match event {
                ProcessEvent::Came(came) => {
                    if self.started.contains_key(&came.instance) {
                        continue;
                    }
                    let (exe, folder) = place(&came.image_path);
                    let first_seen = exe.is_empty() || self.seen.insert(folder.clone());
                    self.timeline.insert((came.at, Kind::Came, came.instance));
                    let name = name_of(came);
                    self.started.insert(
                        came.instance,
                        Started {
                            file: Arc::from(name.to_lowercase()),
                            name,
                            came: came.clone(),
                            folder,
                            first_seen,
                            launched: None,
                        },
                    );
                    if !self.settle_launcher(came.instance) {
                        self.unplaced.push(came.instance);
                    }
                }
                ProcessEvent::Went(went) => {
                    if self.ended.contains_key(&went.instance) {
                        continue;
                    }
                    self.timeline.insert((went.at, Kind::Went, went.instance));
                    self.ended.insert(went.instance, went.clone());
                }
            }
        }
    }

    pub fn take(&mut self, batch: &WindowsProcessEvents) {
        self.lost += u64::from(batch.lost);
        match batch.history_from {
            Some(since) => self.history(since, &batch.events),
            None => self.record(&batch.events),
        }
    }

    pub fn history(&mut self, since: u64, events: &[ProcessEvent]) {
        self.since = Some(self.since.map_or(since, |known| known.min(since)));
        self.record(events);
    }

    pub fn note_running(&mut self, processes: &[WindowsProcessStats]) {
        for process in processes {
            if !process.image_path.is_empty() {
                self.seen.insert(place(&process.image_path).1);
            }
            self.running.insert(
                process.pid,
                Known {
                    name: process.name.clone(),
                    start_time: process.start_time,
                },
            );
        }
        for instance in std::mem::take(&mut self.unplaced) {
            self.settle_launcher(instance);
        }
    }

    fn settle_launcher(&mut self, instance: ProcessInstance) -> bool {
        let Some(started) = self.started.get(&instance) else {
            return true;
        };
        let Some((launcher, name)) = self
            .launched_by(started)
            .and_then(|launcher| self.named(launcher).map(|name| (launcher, Arc::<str>::from(name.to_lowercase()))))
        else {
            return false;
        };
        *self.habits.entry((name.clone(), started.folder.clone())).or_default() += 1;
        if let Some(started) = self.started.get_mut(&instance) {
            started.launched = Some((launcher, name));
        }
        true
    }

    fn routine(&self, started: &Started) -> bool {
        started.launched.as_ref().is_some_and(|(_, name)| {
            self.habits
                .get(&(name.clone(), started.folder.clone()))
                .is_some_and(|&count| count >= Pace::Routine)
        })
    }

    fn hosted(&self, started: &Started) -> bool {
        started.name.eq_ignore_ascii_case(Pace::ConsoleHost) && self.named(started.came.parent).is_some()
    }

    fn has(&self, started: &Started, pick: &Pick) -> bool {
        match pick {
            Pick::Exe(file) => *file == started.file,
            Pick::Folder(folder) => *folder == started.folder,
            Pick::Under(name) => self.under(started.came.parent, name),
        }
    }

    fn under(&self, parent: ProcessInstance, wanted: &str) -> bool {
        let mut next = Some(parent);
        for _ in 0..Pace::ChainDepth {
            let Some(at) = next else {
                return false;
            };
            if self.named(at).is_some_and(|name| name.eq_ignore_ascii_case(wanted)) {
                return true;
            }
            next = self.started.get(&at).map(|started| started.came.parent);
        }
        false
    }

    fn went_has(&self, went: &ProcessWent, pick: &Pick) -> bool {
        if let Some(started) = self.started.get(&went.instance) {
            return self.has(started, pick);
        }
        let (exe, folder) = place(&went.image_path);
        match pick {
            Pick::Exe(wanted) => !exe.is_empty() && **wanted == *file_name(&exe),
            Pick::Folder(wanted) => !folder.is_empty() && *wanted == folder,
            Pick::Under(_) => false,
        }
    }

    fn came_group(&self, groups: &[Group], started: &Started) -> Option<usize> {
        group_of(groups, |pick| self.has(started, pick))
    }

    fn went_group(&self, groups: &[Group], went: &ProcessWent) -> Option<usize> {
        group_of(groups, |pick| self.went_has(went, pick))
    }

    fn began(&self) -> Option<u64> {
        let first = self.timeline.first().map(|(at, ..)| *at);
        match (self.since, first) {
            (Some(since), Some(first)) => Some(since.min(first)),
            (since, first) => since.or(first),
        }
    }

    fn named(&self, instance: ProcessInstance) -> Option<Arc<str>> {
        self.started
            .get(&instance)
            .map(|started| started.name.clone())
            .or_else(|| self.running.get(&instance.pid).map(|known| known.name.clone()))
    }

    fn ancestors<'a>(&'a self, started: &Started) -> impl Iterator<Item = (ProcessInstance, Arc<str>)> + 'a {
        let mut next = Some(started.came.parent);
        std::iter::from_fn(move || {
            let at = next.take()?;
            let name = self.named(at)?;
            if is_shell(&name) {
                next = self.started.get(&at).map(|parent| parent.came.parent);
            }
            Some((at, name))
        })
        .take(Pace::ChainDepth)
    }

    fn launched_by(&self, started: &Started) -> Option<ProcessInstance> {
        self.ancestors(started)
            .last()
            .filter(|(_, name)| !is_shell(name))
            .map(|(at, _)| at)
    }

    fn lineage(&self, started: &Started) -> (Vec<Arc<str>>, Option<ProcessInstance>) {
        let mut chain = vec![started.name.clone()];
        let mut launched_by = None;
        for (at, name) in self.ancestors(started) {
            if !is_shell(&name) {
                launched_by = Some(at);
            }
            chain.push(name);
        }
        chain.reverse();
        (chain, launched_by)
    }
}

pub struct Ask<'a> {
    pub now: u64,
    pub span: Span,
    pub filter: &'a Filter,
    pub groups: &'a [Group],
    pub area: Option<Area>,
    pub clock: fn(u64) -> Clock,
}

struct Frame {
    start: u64,
    end: u64,
}

impl Frame {
    fn of(log: &Log, now: u64, span: Span) -> Self {
        let length = match span {
            Span::HalfMinute => Pace::HalfMinute,
            Span::FiveMinutes => Pace::FiveMinutes,
            Span::Quarter => Pace::Quarter,
            Span::HalfHour => Pace::HalfHour,
            Span::Hour => Pace::Hour,
            Span::Day => Pace::Day,
            Span::Connected => log
                .began()
                .map_or(Pace::Hour, |began| now.saturating_sub(began).max(Ticks::Second)),
        };
        Self {
            start: now.saturating_sub(length),
            end: now + 1,
        }
    }

    fn length(&self) -> u64 {
        self.end - 1 - self.start
    }
}

fn came_lived(log: &Log, started: &Started) -> Lived {
    log.ended
        .get(&started.came.instance)
        .map_or(Lived::Running, |went| Lived::For(went.at.saturating_sub(started.came.at)))
}

fn went_lived(log: &Log, went: &ProcessWent) -> Lived {
    log.started
        .get(&went.instance)
        .map_or(Lived::Unknown, |started| Lived::For(went.at.saturating_sub(started.came.at)))
}

fn scatter(sieve: &Sieve<'_>, frame: &Frame, clock: fn(u64) -> Clock) -> (Scatter, Legend) {
    let log = sieve.log;
    let mut legend = Legend {
        groups: vec![0; sieve.groups.len()],
        other: 0,
    };
    let mut dots = Vec::new();
    let span = (frame.start, Kind::Came, ProcessInstance::default())..(frame.end, Kind::Came, ProcessInstance::default());
    for (at, kind, instance) in log.timeline.range(span) {
        let (graded, lived, faint) = match kind {
            Kind::Came => {
                let Some(started) = log.started.get(instance).filter(|started| !log.hosted(started)) else {
                    continue;
                };
                let graded = sieve
                    .grade_came(started)
                    .or_else(|| log.ended.get(instance).and_then(|went| sieve.grade_went(went)));
                (graded, came_lived(log, started), log.routine(started))
            }
            Kind::Went => {
                if log.started.contains_key(instance) {
                    continue;
                }
                let graded = log.ended.get(instance).and_then(|went| sieve.grade_went(went));
                (graded, Lived::Unknown, false)
            }
        };
        let Some(group) = graded else {
            continue;
        };
        match group {
            Some(group) => legend.groups[group] += 1,
            None => legend.other += 1,
        }
        if sieve.shown(group) {
            dots.push(Dot {
                key: *instance,
                at: *at,
                lived,
                faint,
                hue: sieve.hue(group),
            });
        }
    }
    let scatter = Scatter {
        dots,
        now: frame.end - 1,
        now_clock: clock(frame.end - 1),
        length: frame.length(),
        area: None,
    };
    (scatter, legend)
}

fn exit_of(went: &ProcessWent, since: Option<u64>, clock: fn(u64) -> Clock) -> Exit {
    Exit {
        at: clock(went.at),
        code: went.exit_code,
        lived: since.map_or(0, |since| went.at.saturating_sub(since)),
        cpu_cycles: went.cpu_cycles,
        read_bytes: went.io_read_bytes,
        write_bytes: went.io_write_bytes,
        peak_commit_bytes: went.peak_commit_bytes,
    }
}

fn hue_of(groups: &[Group], group: Option<usize>) -> Option<Hue> {
    group.map(|group| groups[group].hue)
}

fn came_of(log: &Log, groups: &[Group], started: &Started, clock: fn(u64) -> Clock) -> (Came, Option<ProcessInstance>) {
    let came = &started.came;
    let (chain, launched_by) = log.lineage(started);
    let by_parent = launched_by == Some(came.parent) && !came.parent_services.is_empty();
    let launcher = match (&came.scheduled_task, launched_by.and_then(|at| log.named(at))) {
        (Some(task), _) => Launcher::Task(task.clone()),
        (None, Some(_)) if by_parent => Launcher::Services(came.parent_services.clone()),
        (None, Some(name)) => Launcher::Process(name),
        (None, None) => Launcher::Unknown,
    };
    let row = Came {
        key: came.instance,
        at: clock(came.at),
        name: started.name.clone(),
        image_path: came.image_path.clone(),
        command_line: came.command_line.clone(),
        working_dir: came.working_dir.clone(),
        user: came.user.clone(),
        session_id: came.session_id,
        elevated: came.elevated,
        launcher,
        chain,
        parent_services: came.parent_services.clone(),
        first_seen: started.first_seen,
        exit: log
            .ended
            .get(&came.instance)
            .map(|went| exit_of(went, Some(came.at), clock)),
        picks: started.picks(),
        hue: hue_of(groups, log.came_group(groups, started)),
    };
    (row, launched_by)
}

fn went_name(log: &Log, went: &ProcessWent) -> Option<Arc<str>> {
    log.started
        .get(&went.instance)
        .map(|started| started.name.clone())
        .or_else(|| (!went.image_path.is_empty()).then(|| file_name(&went.image_path).into()))
        .or_else(|| (!went.image_name.is_empty()).then(|| went.image_name.clone()))
        .or_else(|| log.named(went.instance))
}

fn went_of(log: &Log, groups: &[Group], went: &ProcessWent, clock: fn(u64) -> Clock) -> Went {
    let started = log
        .started
        .get(&went.instance)
        .map(|started| started.came.at)
        .or(went.started_at)
        .or_else(|| {
            log.running
                .get(&went.instance.pid)
                .map(|known| known.start_time)
                .filter(|start| (1..=went.at).contains(start))
        });
    Went {
        key: went.instance,
        name: went_name(log, went),
        lived: started.map(|start| went.at - start),
        exit: exit_of(went, started, clock),
        hue: hue_of(groups, log.went_group(groups, went)),
    }
}

enum Light {
    Came(ProcessInstance),
    Went(ProcessInstance),
    Series(Vec<(u64, ProcessInstance)>),
}

impl Light {
    fn key(&self) -> ProcessInstance {
        match self {
            Light::Came(instance) | Light::Went(instance) => *instance,
            Light::Series(members) => members.last().map(|(_, instance)| *instance).unwrap_or_default(),
        }
    }
}

fn series_of(
    members: &[(u64, ProcessInstance)],
    log: &Log,
    groups: &[Group],
    clock: fn(u64) -> Clock,
) -> Option<Series> {
    let (at, newest) = members.first()?;
    let newest = log.started.get(newest)?;
    let (launcher, launcher_key) = newest.launched.clone()?;
    let started: Vec<&Started> = members.iter().filter_map(|(_, instance)| log.started.get(instance)).collect();
    let mut names: BTreeMap<Arc<str>, usize> = BTreeMap::new();
    for member in &started {
        *names.entry(member.name.clone()).or_default() += 1;
    }
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut picks = Vec::new();
    if started.iter().all(|member| member.file == newest.file) {
        picks.push(Pick::Exe(newest.file.clone()));
    }
    picks.push(Pick::Folder(newest.folder.clone()));
    picks.push(Pick::Under(launcher_key));
    Some(Series {
        key: members.last()?.1,
        at: clock(*at),
        launcher: log.named(launcher).unwrap_or_default(),
        folder: newest.folder.clone(),
        names,
        count: members.len(),
        went: started.iter().filter(|member| log.ended.contains_key(&member.came.instance)).count(),
        routine: log.routine(newest),
        members: started
            .iter()
            .take(Pace::Shown)
            .map(|member| Rc::new(came_of(log, groups, member, clock).0))
            .collect(),
        picks,
        hue: hue_of(groups, log.came_group(groups, newest)),
    })
}

struct Sieve<'a> {
    log: &'a Log,
    filter: &'a Filter,
    groups: &'a [Group],
    text: String,
    band: Option<(Lived, Lived)>,
}

impl<'a> Sieve<'a> {
    fn new(log: &'a Log, filter: &'a Filter, groups: &'a [Group], area: Option<Area>) -> Self {
        Self {
            log,
            filter,
            groups,
            text: filter.text.trim().to_lowercase(),
            band: area.map(|area| (area.shortest, area.longest)),
        }
    }

    fn held(&self, lived: Lived) -> bool {
        self.band.is_none_or(|(shortest, longest)| (shortest..=longest).contains(&lived))
    }

    fn shown(&self, group: Option<usize>) -> bool {
        group.map_or(self.filter.other, |group| self.groups[group].shown)
    }

    fn grade_came(&self, started: &Started) -> Option<Option<usize>> {
        let passes = self.filter.came
            && (!self.filter.new_only || started.first_seen)
            && !self.log.hosted(started)
            && self.held(came_lived(self.log, started))
            && admitted(self.filter, |pick| self.log.has(started, pick))
            && [&*started.name, &*started.came.command_line, &*started.came.image_path]
                .iter()
                .any(|field| contains(field, &self.text));
        passes.then(|| self.log.came_group(self.groups, started))
    }

    fn grade_went(&self, went: &ProcessWent) -> Option<Option<usize>> {
        let passes = self.filter.went
            && !self.filter.new_only
            && !self.log.started.get(&went.instance).is_some_and(|started| self.log.hosted(started))
            && self.held(went_lived(self.log, went))
            && admitted(self.filter, |pick| self.log.went_has(went, pick))
            && contains(went_name(self.log, went).as_deref().unwrap_or_default(), &self.text);
        passes.then(|| self.log.went_group(self.groups, went))
    }

    fn admits_came(&self, started: &Started) -> bool {
        self.grade_came(started).is_some_and(|group| self.shown(group))
    }

    fn admits_went(&self, went: &ProcessWent) -> bool {
        self.grade_went(went).is_some_and(|group| self.shown(group))
    }

    fn hue(&self, group: Option<usize>) -> Option<Hue> {
        hue_of(self.groups, group)
    }
}

struct Walk<'a> {
    sieve: Sieve<'a>,
    rows: Vec<(u64, Light)>,
    series: HashMap<(ProcessInstance, Arc<str>), usize>,
}

impl Walk<'_> {
    fn came(&mut self, at: u64, instance: ProcessInstance) {
        let sieve = &self.sieve;
        let Some(started) = sieve.log.started.get(&instance).filter(|started| sieve.admits_came(started)) else {
            return;
        };
        let launched = started
            .launched
            .as_ref()
            .filter(|_| sieve.filter.series && !started.folder.is_empty());
        let Some((launcher, _)) = launched else {
            self.rows.push((at, Light::Came(instance)));
            return;
        };
        let key = (*launcher, started.folder.clone());
        match self.series.get(&key) {
            Some(&index) => {
                if let (_, Light::Series(members)) = &mut self.rows[index] {
                    members.push((at, instance));
                }
            }
            None => {
                self.series.insert(key, self.rows.len());
                self.rows.push((at, Light::Series(vec![(at, instance)])));
            }
        }
    }

    fn went(&mut self, at: u64, instance: ProcessInstance) {
        if self.sieve.log.ended.get(&instance).is_some_and(|went| self.sieve.admits_went(went)) {
            self.rows.push((at, Light::Went(instance)));
        }
    }

    fn finish(self) -> Vec<(u64, Light)> {
        let mut rows = Vec::with_capacity(self.rows.len());
        for (at, light) in self.rows {
            match light {
                Light::Series(members) if members.len() < Pace::SeriesLeast => {
                    rows.extend(members.into_iter().map(|(at, instance)| (at, Light::Came(instance))));
                }
                light => rows.push((at, light)),
            }
        }
        rows
    }
}

fn contains(field: &str, text: &str) -> bool {
    text.is_empty() || field.to_lowercase().contains(text)
}

fn row_of(log: &Log, groups: &[Group], row: Light, clock: fn(u64) -> Clock) -> Option<ActivityRow> {
    Some(match row {
        Light::Came(instance) => {
            ActivityRow::Came(Rc::new(came_of(log, groups, log.started.get(&instance)?, clock).0))
        }
        Light::Went(instance) => ActivityRow::Went(Rc::new(went_of(log, groups, log.ended.get(&instance)?, clock))),
        Light::Series(members) => ActivityRow::Series(Rc::new(series_of(&members, log, groups, clock)?)),
    })
}

pub fn row(log: &Log, groups: &[Group], key: ProcessInstance, clock: fn(u64) -> Clock) -> Option<ActivityRow> {
    let light = if log.started.contains_key(&key) {
        Light::Came(key)
    } else {
        Light::Went(key)
    };
    row_of(log, groups, light, clock)
}

pub fn view(log: &Log, ask: &Ask<'_>) -> ActivityView {
    let clock = ask.clock;
    let frame = Frame::of(log, ask.now, ask.span);
    let (scatter, legend) = scatter(&Sieve::new(log, ask.filter, ask.groups, None), &frame, clock);
    let scatter = Scatter {
        area: ask.area,
        ..scatter
    };

    let (from, to) = ask
        .area
        .map_or((frame.start, frame.end), |area| (area.from.max(frame.start), area.to.min(frame.end)));

    let window = |at: u64| (from..to).contains(&at);
    let span = (from, Kind::Came, ProcessInstance::default())..(to, Kind::Came, ProcessInstance::default());
    let (mut came_count, mut went_count) = (0, 0);
    for (_, kind, _) in log.timeline.range(span.clone()) {
        match kind {
            Kind::Came => came_count += 1,
            Kind::Went => went_count += 1,
        }
    }

    let mut walk = Walk {
        sieve: Sieve::new(log, ask.filter, ask.groups, ask.area),
        rows: Vec::new(),
        series: HashMap::new(),
    };
    for (at, kind, instance) in log.timeline.range(span).rev() {
        match kind {
            Kind::Came => walk.came(*at, *instance),
            Kind::Went => {
                let shown = ask.filter.came
                    && log.started.get(instance).is_some_and(|started| window(started.came.at));
                if !shown {
                    walk.went(*at, *instance);
                }
            }
        }
    }
    let mut rows = walk.finish();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.key().cmp(&a.1.key())));
    let earlier = rows.len().saturating_sub(Pace::Shown);
    rows.truncate(Pace::Shown);

    ActivityView {
        scatter: Rc::new(scatter),
        rows: rows
            .into_iter()
            .filter_map(|(_, row)| row_of(log, ask.groups, row, clock))
            .collect(),
        earlier,
        came: came_count,
        went: went_count,
        from: clock(from),
        to: clock(to),
        history_since: log.since.map(clock),
        lost: log.lost,
        legend,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::activity::{ActivityRow, Hue, Launcher};
    use app_contracts::features::agents::{ProcessCame, ProcessInstance, ProcessWent, ScheduledTask};

    use super::*;

    const HOUR: u64 = 60 * Ticks::Minute;
    const BASE: u64 = 1000 * HOUR;

    fn at(minute: u64, second: u64) -> u64 {
        BASE + minute * Ticks::Minute + second * Ticks::Second
    }

    fn utc(at: u64) -> Clock {
        let seconds = at / Ticks::Second;
        Clock {
            hour: (seconds / 3600 % 24) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
        }
    }

    fn id(pid: u32) -> ProcessInstance {
        ProcessInstance {
            pid,
            sequence: u64::from(pid) + 1000,
        }
    }

    fn came(pid: u32, parent: u32, image: &str, when: u64) -> ProcessEvent {
        ProcessEvent::Came(ProcessCame {
            instance: id(pid),
            parent: id(parent),
            at: when,
            image_path: format!(r"C:\bin\{image}").into(),
            command_line: format!("{image} --run").into(),
            ..Default::default()
        })
    }

    fn went(pid: u32, when: u64) -> ProcessEvent {
        ProcessEvent::Went(ProcessWent {
            instance: id(pid),
            at: when,
            exit_code: 3,
            ..Default::default()
        })
    }

    fn running(pid: u32, name: &str, start: u64) -> WindowsProcessStats {
        WindowsProcessStats {
            pid,
            name: name.into(),
            image_path: format!(r"C:\bin\{name}").into(),
            start_time: start,
            ..Default::default()
        }
    }

    fn look(log: &Log, filter: &Filter, area: Option<Area>) -> ActivityView {
        seen(log, filter, &[], area)
    }

    fn seen(log: &Log, filter: &Filter, groups: &[Group], area: Option<Area>) -> ActivityView {
        view(
            log,
            &Ask {
                now: at(59, 59),
                span: Span::Hour,
                filter,
                groups,
                area,
                clock: utc,
            },
        )
    }

    fn group(id: &str, hue: Hue, rules: Vec<Pick>) -> Group {
        Group {
            id: id.into(),
            name: id.into(),
            hue,
            rules,
            shown: true,
        }
    }

    fn hues(view: &ActivityView) -> Vec<(String, Option<Hue>)> {
        let mut hues: Vec<(String, Option<Hue>)> = view
            .rows
            .iter()
            .map(|row| match row {
                ActivityRow::Came(came) => (came.name.to_string(), came.hue),
                other => panic!("a came row, got {other:?}"),
            })
            .collect();
        hues.sort();
        hues
    }

    fn dot_hues(view: &ActivityView) -> Vec<(u32, Option<Hue>)> {
        let mut hues: Vec<(u32, Option<Hue>)> = view.scatter.dots.iter().map(|dot| (dot.key.pid, dot.hue)).collect();
        hues.sort();
        hues
    }

    fn singles() -> Filter {
        Filter {
            series: false,
            ..Filter::default()
        }
    }

    #[test]
    fn a_process_takes_the_hue_of_the_first_group_it_falls_in_on_the_list_and_the_chart() {
        let mut log = Log::default();
        log.record(&[
            came_at(20, 1, r"C:\Bin\git.exe", at(10, 0)),
            came_at(21, 1, r"C:\Bin\other.exe", at(11, 0)),
            came_at(22, 1, r"D:\Tools\Git.exe", at(12, 0)),
            came_at(23, 1, r"C:\Else\else.exe", at(13, 0)),
        ]);
        let groups = [
            group("tooling", Hue::Purple, vec![Pick::Exe("git.exe".into())]),
            group("bin", Hue::Teal, vec![Pick::Folder(r"c:\bin".into())]),
        ];

        let view = seen(&log, &singles(), &groups, None);

        assert_eq!(
            hues(&view),
            [
                ("Git.exe".into(), Some(Hue::Purple)),
                ("else.exe".into(), None),
                ("git.exe".into(), Some(Hue::Purple)),
                ("other.exe".into(), Some(Hue::Teal)),
            ]
        );
        assert_eq!(
            dot_hues(&view),
            [(20, Some(Hue::Purple)), (21, Some(Hue::Teal)), (22, Some(Hue::Purple)), (23, None)]
        );
    }

    #[test]
    fn everything_under_a_program_is_in_its_group_however_deep_but_not_the_program() {
        let mut log = Log::default();
        log.record(&[
            came_at(4, 1, r"C:\Tools\claude.exe", at(5, 0)),
            came_at(20, 4, r"C:\Git\bin\bash.exe", at(10, 0)),
            came_at(21, 20, r"C:\Rust\cargo.exe", at(11, 0)),
            came_at(22, 21, r"C:\Rust\rustc.exe", at(12, 0)),
            came_at(30, 1, r"C:\Rust\rustc.exe", at(13, 0)),
        ]);
        let groups = [group("tooling", Hue::Purple, vec![Pick::Under("claude.exe".into())])];

        let view = seen(&log, &singles(), &groups, None);

        assert_eq!(
            dot_hues(&view),
            [
                (4, None),
                (20, Some(Hue::Purple)),
                (21, Some(Hue::Purple)),
                (22, Some(Hue::Purple)),
                (30, None),
            ]
        );
    }

    #[test]
    fn a_hidden_group_leaves_the_list_and_the_chart_and_the_legend_still_counts_it() {
        let mut log = Log::default();
        log.record(&[
            came_at(20, 1, r"C:\Windows\System32\taskhostw.exe", at(10, 0)),
            came_at(21, 1, r"C:\Windows\System32\taskhostw.exe", at(11, 0)),
            came_at(22, 1, r"C:\Apps\app.exe", at(12, 0)),
        ]);
        let groups = [Group {
            shown: false,
            ..Group::windows_background()
        }];

        let view = seen(&log, &singles(), &groups, None);

        assert_eq!(hues(&view), [("app.exe".into(), None)]);
        assert_eq!(dot_hues(&view), [(22, None)]);
        assert_eq!(
            view.legend,
            Legend {
                groups: vec![2],
                other: 1
            }
        );
    }

    #[test]
    fn with_the_rest_hidden_only_grouped_processes_are_left() {
        let mut log = Log::default();
        log.record(&[
            came_at(20, 1, r"C:\Windows\System32\taskhostw.exe", at(10, 0)),
            came_at(22, 1, r"C:\Apps\app.exe", at(12, 0)),
        ]);
        let filter = Filter {
            other: false,
            ..singles()
        };

        let view = seen(&log, &filter, &[Group::windows_background()], None);

        assert_eq!(hues(&view), [("taskhostw.exe".into(), Some(Hue::Teal))]);
        assert_eq!(dot_hues(&view), [(20, Some(Hue::Teal))]);
    }

    #[test]
    fn a_program_is_hidden_by_its_name_wherever_it_lives_and_what_runs_under_it_too() {
        let mut log = Log::default();
        log.record(&[
            came_at(20, 1, r"C:\Rust\1.80\rustc.exe", at(10, 0)),
            came_at(21, 1, r"C:\Rust\1.81\rustc.exe", at(11, 0)),
            came_at(4, 1, r"C:\Tools\claude.exe", at(12, 0)),
            came_at(30, 4, r"C:\Git\bin\bash.exe", at(13, 0)),
            came_at(31, 30, r"C:\Git\bin\git.exe", at(14, 0)),
        ]);
        let filter = Filter {
            hidden: vec![Pick::Exe("rustc.exe".into()), Pick::Under("claude.exe".into())],
            ..singles()
        };

        assert_eq!(hues(&look(&log, &filter, None)), [("claude.exe".into(), None)]);
    }

    fn rows(log: &Log) -> Vec<ActivityRow> {
        look(log, &Filter::default(), None).rows
    }

    fn first(rows: &[ActivityRow]) -> &ActivityRow {
        assert!(!rows.is_empty(), "no rows");
        &rows[0]
    }

    fn only_came(row: &ActivityRow) -> &app_contracts::features::activity::Came {
        match row {
            ActivityRow::Came(came) => came,
            other => panic!("a came row, got {other:?}"),
        }
    }

    #[test]
    fn each_span_lists_what_came_within_it() {
        let now = BASE + 30 * HOUR;
        let mut log = Log::default();
        log.record(&[
            came(7, 0, "g.exe", now - 25 * HOUR),
            came(6, 0, "f.exe", now - 23 * HOUR),
            came(5, 0, "e.exe", now - 59 * Ticks::Minute),
            came(4, 0, "d.exe", now - 29 * Ticks::Minute),
            came(3, 0, "c.exe", now - 14 * Ticks::Minute),
            came(2, 0, "b.exe", now - 4 * Ticks::Minute),
            came(1, 0, "a.exe", now - 20 * Ticks::Second),
        ]);
        let filter = Filter::default();
        let listed = |span| {
            view(
                &log,
                &Ask {
                    now,
                    span,
                    filter: &filter,
                    groups: &[],
                    area: None,
                    clock: utc,
                },
            )
            .rows
            .len()
        };

        let spans = [
            Span::HalfMinute,
            Span::FiveMinutes,
            Span::Quarter,
            Span::HalfHour,
            Span::Hour,
            Span::Day,
            Span::Connected,
        ];
        assert_eq!(spans.map(listed), [1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn a_process_that_came_and_went_is_one_row_that_knows_its_exit() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "tool.exe", at(10, 0)), went(20, at(10, 2))]);

        let rows = rows(&log);
        assert_eq!(rows.len(), 1, "{rows:#?}");
        let row = only_came(&rows[0]);
        assert_eq!(&*row.name, "tool.exe");
        assert_eq!(row.at, utc(at(10, 0)));
        let exit = row.exit.as_ref().expect("it went");
        assert_eq!(exit.code, 3);
        assert_eq!(exit.lived, 2 * Ticks::Second);
    }

    #[test]
    fn a_process_that_went_without_a_start_here_is_named_from_what_was_running() {
        let mut log = Log::default();
        log.note_running(&[running(30, "OneDrive.exe", at(0, 0) - 2 * HOUR)]);
        log.record(&[went(30, at(45, 0))]);

        let rows = rows(&log);
        let [ActivityRow::Went(row)] = rows.as_slice() else {
            panic!("one went row: {rows:#?}");
        };
        assert_eq!(row.name.as_deref(), Some("OneDrive.exe"));
        assert_eq!(row.lived, Some(2 * HOUR + 45 * Ticks::Minute));
    }

    #[test]
    fn a_process_never_seen_before_it_went_has_no_name_rather_than_a_number() {
        let mut log = Log::default();
        log.record(&[went(47, at(45, 0))]);

        let rows = rows(&log);
        let [ActivityRow::Went(row)] = rows.as_slice() else {
            panic!("one went row: {rows:#?}");
        };
        assert_eq!(row.name, None);
        assert_eq!(row.lived, None);
    }

    #[test]
    fn an_exit_from_before_the_history_is_named_by_what_the_service_says_about_it() {
        let mut log = Log::default();
        log.record(&[
            ProcessEvent::Went(ProcessWent {
                instance: id(40),
                at: at(45, 0),
                image_path: r"C:\Windows\System32\cmd.exe".into(),
                image_name: "cmd.exe".into(),
                started_at: Some(at(44, 0)),
                ..Default::default()
            }),
            ProcessEvent::Went(ProcessWent {
                instance: id(41),
                at: at(46, 0),
                image_name: "SearchProtocol".into(),
                ..Default::default()
            }),
        ]);

        let rows = rows(&log);
        let [ActivityRow::Went(short), ActivityRow::Went(full)] = rows.as_slice() else {
            panic!("two went rows: {rows:#?}");
        };
        assert_eq!(full.name.as_deref(), Some("cmd.exe"));
        assert_eq!(full.lived, Some(Ticks::Minute));
        assert_eq!(short.name.as_deref(), Some("SearchProtocol"));
        assert_eq!(short.lived, None);
    }

    #[test]
    fn only_the_newest_rows_are_listed_and_the_rest_are_counted() {
        let mut log = Log::default();
        let starts: Vec<_> = (0..250).map(|n| came(1000 + n, 1, "a.exe", at(10, 0) + u64::from(n) * Ticks::Second)).collect();
        log.record(&starts);

        let view = look(&log, &Filter::default(), None);
        assert_eq!((view.rows.len(), view.earlier), (200, 50));
        assert_eq!(view.rows[0].key(), id(1249));
    }

    #[test]
    fn rows_are_newest_first() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(5, 0)), came(21, 1, "b.exe", at(6, 0))]);

        let names: Vec<_> = rows(&log).iter().map(|row| only_came(row).name.clone()).collect();
        assert_eq!(names, [Arc::from("b.exe"), Arc::from("a.exe")]);
    }

    #[test]
    fn the_chain_climbs_through_shells_to_the_first_that_is_not_one() {
        let mut log = Log::default();
        log.note_running(&[running(4, "explorer.exe", at(0, 0))]);
        log.record(&[
            came(20, 4, "cmd.exe", at(10, 0)),
            came(21, 20, "powershell.exe", at(10, 1)),
        ]);

        let rows = rows(&log);
        let shell = only_came(first(&rows));
        assert_eq!(&*shell.name, "powershell.exe");
        assert_eq!(
            shell.chain,
            [Arc::from("explorer.exe"), Arc::from("cmd.exe"), Arc::from("powershell.exe")]
        );
        assert_eq!(shell.launcher, Launcher::Process("explorer.exe".into()));
    }

    #[test]
    fn a_scheduled_task_is_the_launcher() {
        let mut log = Log::default();
        let task = ScheduledTask {
            name: "Vendor Updater Daily".into(),
            path: r"\Vendor\Updater".into(),
        };
        let ProcessEvent::Came(mut started) = came(20, 9, "powershell.exe", at(10, 0)) else {
            unreachable!()
        };
        started.scheduled_task = Some(task.clone());
        log.record(&[ProcessEvent::Came(started)]);

        assert_eq!(only_came(first(&rows(&log))).launcher, Launcher::Task(task));
    }

    fn git_burst(log: &mut Log, starts: u32) {
        log.note_running(&[running(4, "Code.exe", at(0, 0))]);
        for n in 0..starts {
            let shell = 100 + n * 2;
            let when = at(20, 0) + u64::from(n) * Ticks::Second / 4;
            log.record(&[
                came(shell, 4, "cmd.exe", when),
                came(shell + 1, shell, "git.exe", when + 1),
                went(shell + 1, when + 2),
                went(shell, when + 3),
            ]);
        }
    }

    fn came_at(pid: u32, parent: u32, path: &str, when: u64) -> ProcessEvent {
        ProcessEvent::Came(ProcessCame {
            instance: id(pid),
            parent: id(parent),
            at: when,
            image_path: path.into(),
            command_line: format!("\"{path}\"").into(),
            ..Default::default()
        })
    }

    fn only_series(rows: &[ActivityRow]) -> &app_contracts::features::activity::Series {
        match rows {
            [ActivityRow::Series(series), ..] => series,
            other => panic!("a series first: {other:#?}"),
        }
    }

    const GIT: &str = r"C:\Program Files\Git\cmd\git.exe";

    #[test]
    fn quick_starts_from_one_launcher_and_folder_fold_into_a_series() {
        let mut log = Log::default();
        git_burst(&mut log, 5);

        let rows = rows(&log);
        assert_eq!(rows.len(), 1, "{rows:#?}");
        let series = only_series(&rows);
        assert_eq!(&*series.launcher, "Code.exe");
        assert_eq!((series.count, series.members.len(), series.went), (10, 10, 10));
        assert_eq!(series.names, [(Arc::from("cmd.exe"), 5), (Arc::from("git.exe"), 5)]);
    }

    #[test]
    fn repeats_from_one_launcher_and_folder_are_one_series_however_far_apart() {
        let mut log = Log::default();
        log.note_running(&[running(4, "claude.exe", at(0, 0))]);
        log.record(&[
            came_at(20, 4, GIT, at(5, 0)),
            came_at(21, 4, GIT, at(20, 0)),
            came_at(22, 4, r"C:\Tools\rg.exe", at(30, 0)),
            came_at(23, 4, GIT, at(40, 0)),
        ]);

        let rows = rows(&log);
        assert_eq!(rows.len(), 2, "{rows:#?}");
        let series = only_series(&rows);
        assert_eq!((series.count, series.at), (3, utc(at(40, 0))));
        assert_eq!(&*series.folder, r"c:\program files\git\cmd");
        assert_eq!(&*only_came(&rows[1]).name, "rg.exe");
    }

    #[test]
    fn programs_with_different_names_from_one_folder_are_one_series() {
        let mut log = Log::default();
        log.note_running(&[running(4, "cargo.exe", at(0, 0))]);
        let deps = |name: &str| format!(r"C:\proj\target\debug\deps\{name}");
        log.record(&[
            came_at(20, 4, &deps("ui-1f2e.exe"), at(10, 0)),
            came_at(21, 4, &deps("domain-3d4c.exe"), at(10, 1)),
            came_at(22, 4, &deps("desktop-5b6a.exe"), at(10, 2)),
        ]);

        let rows = rows(&log);
        assert_eq!(rows.len(), 1, "{rows:#?}");
        assert_eq!(only_series(&rows).names.len(), 3);
    }

    #[test]
    fn two_starts_are_not_a_series() {
        let mut log = Log::default();
        log.note_running(&[running(4, "Code.exe", at(0, 0))]);
        log.record(&[came(20, 4, "git.exe", at(20, 0)), came(21, 4, "git.exe", at(20, 1))]);

        let rows = rows(&log);
        assert!(
            rows.len() == 2 && rows.iter().all(|row| matches!(row, ActivityRow::Came(_))),
            "{rows:#?}"
        );
    }

    #[test]
    fn with_series_off_repeats_are_listed_one_by_one() {
        let mut log = Log::default();
        git_burst(&mut log, 3);
        let filter = Filter {
            series: false,
            ..Filter::default()
        };

        let rows = look(&log, &filter, None).rows;
        assert!(
            rows.len() == 6 && rows.iter().all(|row| matches!(row, ActivityRow::Came(_))),
            "{rows:#?}"
        );
    }

    #[test]
    fn a_console_host_hides_in_its_owner() {
        let mut log = Log::default();
        log.note_running(&[running(4, "claude.exe", at(0, 0))]);
        log.record(&[
            came_at(20, 4, GIT, at(10, 0)),
            came_at(21, 20, r"C:\Windows\System32\conhost.exe", at(10, 0) + 1),
        ]);

        let view = look(&log, &Filter::default(), None);
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(&*only_came(&view.rows[0]).name, "git.exe");
        assert_eq!(view.scatter.dots.len(), 1, "{:#?}", view.scatter);
    }

    #[test]
    fn first_seen_is_per_folder_so_a_rebuilt_binary_is_not_new() {
        let mut log = Log::default();
        log.record(&[
            came_at(20, 1, r"C:\proj\target\debug\deps\ui-1f2e.exe", at(10, 0)),
            came_at(21, 1, r"C:\proj\target\debug\deps\ui-9a8b.exe", at(11, 0)),
            came_at(22, 1, r"C:\Temp\x1\setup.exe", at(12, 0)),
        ]);

        let rows = rows(&log);
        let flags: Vec<bool> = rows.iter().map(|row| only_came(row).first_seen).collect();
        assert_eq!(flags, [true, false, true]);
    }

    #[test]
    fn a_launcher_and_folder_seen_twenty_times_is_routine_and_faint_on_the_chart() {
        let mut log = Log::default();
        log.note_running(&[running(4, "claude.exe", at(0, 0))]);
        let starts: Vec<_> = (0..20).map(|n| came_at(100 + n, 4, GIT, at(10, u64::from(n)))).collect();
        log.record(&starts);
        log.record(&[came_at(20, 4, r"C:\Tools\rg.exe", at(30, 0))]);

        let view = look(&log, &Filter::default(), None);
        assert!(dot(&view.scatter, 100).faint && dot(&view.scatter, 119).faint, "{:#?}", view.scatter);
        assert!(!dot(&view.scatter, 20).faint);
        let series = view.rows.iter().find_map(|row| match row {
            ActivityRow::Series(series) => Some(series.clone()),
            _ => None,
        });
        assert!(series.is_some_and(|series| series.routine), "{:#?}", view.rows);
    }

    #[test]
    fn only_one_exe_or_hiding_a_folder_narrows_the_list_and_fades_the_rest() {
        let mut log = Log::default();
        log.record(&[came_at(20, 1, r"C:\A\a.exe", at(10, 0)), came_at(21, 1, r"C:\B\b.exe", at(11, 0))]);
        let a = Pick::Exe("a.exe".into());

        let rows = rows(&log);
        let picks = &only_came(&rows[1]).picks;
        assert_eq!(picks, &[a.clone(), Pick::Folder(r"c:\a".into())]);

        let only = Filter {
            only: Some(a),
            ..Filter::default()
        };
        let view = look(&log, &only, None);
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(&*only_came(&view.rows[0]).name, "a.exe");

        let hidden = Filter {
            hidden: vec![Pick::Folder(r"c:\a".into())],
            ..Filter::default()
        };
        let view = look(&log, &hidden, None);
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(&*only_came(&view.rows[0]).name, "b.exe");
    }

    fn drawn(scatter: &Scatter) -> Vec<u32> {
        let mut pids: Vec<u32> = scatter.dots.iter().map(|dot| dot.key.pid).collect();
        pids.sort_unstable();
        pids
    }

    #[test]
    fn what_the_list_leaves_out_the_chart_leaves_out_too() {
        let mut log = Log::default();
        log.note_running(&[running(30, "old.exe", at(0, 0))]);
        log.record(&[
            came_at(20, 1, r"C:\A\a.exe", at(10, 0)),
            came_at(21, 1, r"C:\B\b.exe", at(11, 0)),
            went(30, at(12, 0)),
        ]);
        let drawn_with = |filter: Filter| drawn(&look(&log, &filter, None).scatter);

        assert_eq!(drawn_with(Filter::default()), [20, 21, 30]);
        assert_eq!(
            drawn_with(Filter {
                hidden: vec![Pick::Folder(r"c:\a".into())],
                ..Filter::default()
            }),
            [21, 30]
        );
        assert_eq!(
            drawn_with(Filter {
                only: Some(Pick::Exe("a.exe".into())),
                ..Filter::default()
            }),
            [20]
        );
        assert_eq!(
            drawn_with(Filter {
                went: false,
                ..Filter::default()
            }),
            [20, 21]
        );
        assert_eq!(
            drawn_with(Filter {
                came: false,
                ..Filter::default()
            }),
            [30]
        );
        assert_eq!(
            drawn_with(Filter {
                text: "b.exe".into(),
                ..Filter::default()
            }),
            [21]
        );
    }

    #[test]
    fn a_selected_area_narrows_the_list_and_leaves_the_chart_whole() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0)), came(21, 1, "b.exe", at(40, 0))]);
        let area = Area {
            from: at(5, 0),
            to: at(15, 0),
            shortest: Lived::Unknown,
            longest: Lived::Running,
        };

        let view = look(&log, &Filter::default(), Some(area));

        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(drawn(&view.scatter), [20, 21]);
    }

    #[test]
    fn came_and_went_can_each_be_hidden() {
        let mut log = Log::default();
        log.note_running(&[running(30, "old.exe", at(0, 0))]);
        log.record(&[came(20, 1, "new.exe", at(10, 0)), went(30, at(11, 0))]);

        let came_only = Filter {
            went: false,
            ..Filter::default()
        };
        let rows = look(&log, &came_only, None).rows;
        assert!(matches!(rows.as_slice(), [ActivityRow::Came(_)]), "{rows:#?}");

        let went_only = Filter {
            came: false,
            ..Filter::default()
        };
        let rows = look(&log, &went_only, None).rows;
        assert!(matches!(rows.as_slice(), [ActivityRow::Went(_)]), "{rows:#?}");
    }

    #[test]
    fn with_starts_hidden_an_exit_is_listed_even_when_its_start_was_in_range() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "tool.exe", at(10, 0)), went(20, at(10, 2))]);
        let went_only = Filter {
            came: false,
            ..Filter::default()
        };

        let rows = look(&log, &went_only, None).rows;
        let [ActivityRow::Went(row)] = rows.as_slice() else {
            panic!("one went row: {rows:#?}");
        };
        assert_eq!(row.lived, Some(2 * Ticks::Second));
    }

    #[test]
    fn the_search_matches_the_command_line_ignoring_case() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0)), came(21, 1, "b.exe", at(10, 30))]);
        let filter = Filter {
            text: "A.EXE --R".into(),
            ..Filter::default()
        };

        let rows = look(&log, &filter, None).rows;
        assert_eq!(rows.len(), 1, "{rows:#?}");
        assert_eq!(&*only_came(&rows[0]).name, "a.exe");
    }

    #[test]
    fn an_exe_never_seen_running_is_first_seen() {
        let mut log = Log::default();
        log.note_running(&[running(4, "known.exe", at(0, 0))]);
        log.record(&[
            came(20, 1, "known.exe", at(10, 0)),
            came_at(21, 1, r"C:\Temp\x1\setup.exe", at(11, 0)),
            came_at(22, 1, r"C:\Temp\x1\setup.exe", at(12, 0)),
        ]);

        let rows = rows(&log);
        let flags: Vec<_> = rows.iter().map(|row| (only_came(row).name.clone(), only_came(row).first_seen)).collect();
        assert_eq!(
            flags,
            [
                (Arc::from("setup.exe"), false),
                (Arc::from("setup.exe"), true),
                (Arc::from("known.exe"), false),
            ]
        );

        let filter = Filter {
            new_only: true,
            ..Filter::default()
        };
        assert_eq!(look(&log, &filter, None).rows.len(), 1);
    }

    fn dot(scatter: &Scatter, pid: u32) -> Dot {
        let found = scatter.dots.iter().find(|dot| dot.key == id(pid));
        assert!(found.is_some(), "no dot for {pid}: {scatter:#?}");
        *found.unwrap()
    }

    #[test]
    fn each_process_of_the_hour_is_a_dot_placed_by_when_it_came_and_how_long_it_lived() {
        let mut log = Log::default();
        log.record(&[
            came(20, 1, "short.exe", at(15, 0)),
            went(20, at(15, 1)),
            came(21, 1, "long.exe", at(30, 0)),
            went(21, at(40, 0)),
            came(22, 1, "alive.exe", at(45, 0)),
            went(30, at(50, 0)),
        ]);

        let scatter = look(&log, &Filter::default(), None).scatter;
        assert_eq!(scatter.dots.len(), 4, "{scatter:#?}");
        assert_eq!((scatter.now, scatter.length), (at(59, 59), HOUR));
        assert_eq!(scatter.now_clock, utc(at(59, 59)));
        let (short, long, alive) = (dot(&scatter, 20), dot(&scatter, 21), dot(&scatter, 22));
        assert_eq!((short.at, long.at, alive.at), (at(15, 0), at(30, 0), at(45, 0)));
        assert_eq!(short.lived, Lived::For(Ticks::Second));
        assert_eq!(long.lived, Lived::For(10 * Ticks::Minute));
        assert_eq!(alive.lived, Lived::Running);
        let orphan = dot(&scatter, 30);
        assert_eq!((orphan.at, orphan.lived), (at(50, 0), Lived::Unknown));
    }

    #[test]
    fn a_hovered_dot_is_its_full_row() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "tool.exe", at(10, 0)), went(20, at(10, 2)), went(30, at(20, 0))]);

        match row(&log, &[], id(20), utc) {
            Some(ActivityRow::Came(came)) => {
                assert_eq!(&*came.name, "tool.exe");
                assert!(came.exit.is_some(), "{came:#?}");
            }
            other => panic!("a came row: {other:#?}"),
        }
        assert!(matches!(row(&log, &[], id(30), utc), Some(ActivityRow::Went(_))));
        assert_eq!(row(&log, &[], id(99), utc), None);
    }

    #[test]
    fn an_area_keeps_only_the_processes_inside_it() {
        let mut log = Log::default();
        log.record(&[
            came(20, 1, "short.exe", at(15, 0)),
            went(20, at(15, 1)),
            came(21, 1, "long.exe", at(16, 0)),
            went(21, at(26, 0)),
            came(22, 1, "later.exe", at(40, 0)),
            went(22, at(40, 1)),
        ]);
        let area = Area {
            from: at(10, 0),
            to: at(20, 0),
            shortest: Lived::For(0),
            longest: Lived::For(2 * Ticks::Second),
        };

        let view = look(&log, &Filter::default(), Some(area));

        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(&*only_came(&view.rows[0]).name, "short.exe");
        assert_eq!(view.scatter.area, Some(area));
    }

    #[test]
    fn a_batch_with_a_start_of_history_says_where_history_starts() {
        let mut log = Log::default();
        log.take(&WindowsProcessEvents {
            history_from: Some(at(5, 0)),
            events: Arc::from([came(20, 1, "a.exe", at(10, 0))]),
            lost: 0,
        });
        log.take(&WindowsProcessEvents {
            history_from: None,
            events: Arc::from([went(20, at(10, 1))]),
            lost: 0,
        });

        let view = look(&log, &Filter::default(), None);
        assert_eq!(view.history_since, Some(utc(at(5, 0))));
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
    }

    #[test]
    fn missed_events_add_up() {
        let mut log = Log::default();
        for lost in [3, 2] {
            log.take(&WindowsProcessEvents {
                lost,
                ..WindowsProcessEvents::default()
            });
        }

        assert_eq!(look(&log, &Filter::default(), None).lost, 5);
    }

    #[test]
    fn a_parent_hosting_services_launches_by_its_services() {
        let mut log = Log::default();
        log.note_running(&[running(4, "svchost.exe", at(0, 0))]);
        let ProcessEvent::Came(mut started) = came(20, 4, "taskhostw.exe", at(10, 0)) else {
            unreachable!()
        };
        started.parent_services = Arc::from([Arc::from("Schedule")]);
        log.record(&[ProcessEvent::Came(started)]);

        assert_eq!(
            only_came(first(&rows(&log))).launcher,
            Launcher::Services(Arc::from([Arc::from("Schedule")]))
        );
    }

    #[test]
    fn history_after_a_reconnect_does_not_repeat_what_is_known() {
        let mut log = Log::default();
        let events = [came(20, 1, "a.exe", at(10, 0)), went(20, at(10, 1))];
        log.record(&events);
        log.history(at(5, 0), &events);

        assert_eq!(rows(&log).len(), 1);
        assert_eq!(look(&log, &Filter::default(), None).history_since, Some(utc(at(5, 0))));
    }
}
