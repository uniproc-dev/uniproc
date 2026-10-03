use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use app_contracts::features::activity::{
    ActivityRow, ActivityView, Bucket, Burst, Came, Clock, Exit, Filter, Histogram, Launcher, Span, Went,
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
    const Buckets: u64 = 60;
    const Quarter: u64 = 15 * Ticks::Minute;
    const Hour: u64 = 60 * Ticks::Minute;
    const BurstGap: u64 = 2 * Ticks::Second;
    const BurstLeast: usize = 3;
    const ChainDepth: usize = 16;
    const Shown: usize = 200;
    const Shells: [&str; 5] = ["cmd.exe", "powershell.exe", "pwsh.exe", "wscript.exe", "cscript.exe"];
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Came,
    Went,
}

struct Started {
    came: ProcessCame,
    name: Arc<str>,
    first_seen: bool,
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
    seen: HashSet<String>,
    running: HashMap<u32, Known>,
    since: Option<u64>,
    lost: u64,
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
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
                    let first_seen = came.image_path.is_empty() || self.seen.insert(came.image_path.to_lowercase());
                    self.timeline.insert((came.at, Kind::Came, came.instance));
                    self.started.insert(
                        came.instance,
                        Started {
                            name: name_of(came),
                            came: came.clone(),
                            first_seen,
                        },
                    );
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
                self.seen.insert(process.image_path.to_lowercase());
            }
            self.running.insert(
                process.pid,
                Known {
                    name: process.name.clone(),
                    start_time: process.start_time,
                },
            );
        }
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
    pub range: Option<(u64, u64)>,
    pub clock: fn(u64) -> Clock,
}

struct Frame {
    start: u64,
    width: u64,
}

impl Frame {
    fn of(log: &Log, now: u64, span: Span) -> Self {
        let length = match span {
            Span::Quarter => Pace::Quarter,
            Span::Hour => Pace::Hour,
            Span::Connected => log
                .began()
                .map_or(Pace::Hour, |began| now.saturating_sub(began).max(Ticks::Second)),
        };
        let width = length.div_ceil(Pace::Buckets).max(1);
        let end = now.div_ceil(width) * width;
        Self {
            start: end.saturating_sub(width * Pace::Buckets),
            width,
        }
    }

    fn end(&self) -> u64 {
        self.start + self.width * Pace::Buckets
    }

    fn bucket(&self, at: u64) -> Option<usize> {
        (self.start..self.end())
            .contains(&at)
            .then(|| ((at - self.start) / self.width) as usize)
    }
}

pub fn bucket_range(log: &Log, now: u64, span: Span, index: usize) -> Option<(u64, u64)> {
    let frame = Frame::of(log, now, span);
    (index < Pace::Buckets as usize).then(|| {
        let from = frame.start + frame.width * index as u64;
        (from, from + frame.width)
    })
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

fn came_of(log: &Log, started: &Started, clock: fn(u64) -> Clock) -> (Came, Option<ProcessInstance>) {
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
    };
    (row, launched_by)
}

fn went_of(log: &Log, went: &ProcessWent, clock: fn(u64) -> Clock) -> Went {
    let started = log.started.get(&went.instance).map(|started| started.came.at).or_else(|| {
        log.running
            .get(&went.instance.pid)
            .map(|known| known.start_time)
            .filter(|start| (1..=went.at).contains(start))
    });
    Went {
        key: went.instance,
        name: log.named(went.instance),
        lived: started.map(|start| went.at - start),
        exit: exit_of(went, started, clock),
    }
}

struct Group {
    launcher: ProcessInstance,
    first: u64,
    last: u64,
    members: Vec<(u64, ProcessInstance)>,
}

enum Light {
    Came(ProcessInstance),
    Went(ProcessInstance),
    Burst(Group),
}

impl Light {
    fn key(&self) -> ProcessInstance {
        match self {
            Light::Came(instance) | Light::Went(instance) => *instance,
            Light::Burst(group) => group.members[0].1,
        }
    }
}

fn burst_of(group: Group, log: &Log, clock: fn(u64) -> Clock) -> Burst {
    let members: Vec<Rc<Came>> = group
        .members
        .iter()
        .filter_map(|(_, instance)| log.started.get(instance))
        .map(|started| Rc::new(came_of(log, started, clock).0))
        .collect();
    let mut names: BTreeMap<Arc<str>, usize> = BTreeMap::new();
    for came in &members {
        *names.entry(came.name.clone()).or_default() += 1;
    }
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let ended = group
        .members
        .iter()
        .filter_map(|(_, instance)| log.ended.get(instance).map(|went| went.at))
        .max()
        .unwrap_or(group.last);
    Burst {
        key: group.members[0].1,
        at: clock(group.first),
        launcher: log.named(group.launcher).unwrap_or_default(),
        names,
        went: members.iter().filter(|came| came.exit.is_some()).count(),
        lasted: ended.max(group.last) - group.first,
        members,
    }
}

struct Walk<'a> {
    log: &'a Log,
    filter: &'a Filter,
    text: String,
    rows: Vec<(u64, Light)>,
    open: HashMap<ProcessInstance, Group>,
}

impl Walk<'_> {
    fn keep(&mut self, at: u64, row: Light) {
        if kept(self.log, &row, self.filter, &self.text) {
            self.rows.push((at, row));
        }
    }

    fn close(&mut self, mut group: Group) {
        group.members.reverse();
        if group.members.len() >= Pace::BurstLeast {
            self.keep(group.first, Light::Burst(group));
        } else {
            for (at, came) in group.members {
                self.keep(at, Light::Came(came));
            }
        }
    }

    fn came(&mut self, at: u64, came: ProcessInstance, launcher: Option<ProcessInstance>) {
        let Some(launcher) = launcher else {
            self.keep(at, Light::Came(came));
            return;
        };
        match self.open.get_mut(&launcher) {
            Some(group) if group.first - at <= Pace::BurstGap => {
                group.first = at;
                group.members.push((at, came));
            }
            _ => {
                let group = Group {
                    launcher,
                    first: at,
                    last: at,
                    members: vec![(at, came)],
                };
                if let Some(done) = self.open.insert(launcher, group) {
                    self.close(done);
                }
            }
        }
    }

    fn settle(&mut self, at: u64) {
        if self.open.values().all(|group| group.first - at <= Pace::BurstGap) {
            return;
        }
        let (done, open): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.open).into_values().partition(|group| group.first - at > Pace::BurstGap);
        self.open = open.into_iter().map(|group| (group.launcher, group)).collect();
        for group in done {
            self.close(group);
        }
    }

    fn enough(&self) -> bool {
        self.open.is_empty() && self.rows.len() >= Pace::Shown
    }

    fn finish(mut self) -> Vec<(u64, Light)> {
        for group in std::mem::take(&mut self.open).into_values().collect::<Vec<_>>() {
            self.close(group);
        }
        self.rows
    }
}

fn contains(field: &str, text: &str) -> bool {
    text.is_empty() || field.to_lowercase().contains(text)
}

fn came_kept(log: &Log, instance: &ProcessInstance, filter: &Filter, text: &str) -> bool {
    log.started.get(instance).is_some_and(|started| {
        (!filter.new_only || started.first_seen)
            && [&*started.name, &*started.came.command_line, &*started.came.image_path]
                .iter()
                .any(|field| contains(field, text))
    })
}

fn kept(log: &Log, row: &Light, filter: &Filter, text: &str) -> bool {
    match row {
        Light::Came(instance) => filter.came && came_kept(log, instance, filter, text),
        Light::Went(instance) => {
            filter.went && !filter.new_only && contains(log.named(*instance).as_deref().unwrap_or_default(), text)
        }
        Light::Burst(group) => {
            filter.came
                && filter.bursts
                && group.members.iter().any(|(_, instance)| came_kept(log, instance, filter, text))
        }
    }
}

fn row_of(log: &Log, row: Light, clock: fn(u64) -> Clock) -> Option<ActivityRow> {
    Some(match row {
        Light::Came(instance) => ActivityRow::Came(Rc::new(came_of(log, log.started.get(&instance)?, clock).0)),
        Light::Went(instance) => ActivityRow::Went(Rc::new(went_of(log, log.ended.get(&instance)?, clock))),
        Light::Burst(group) => ActivityRow::Burst(Rc::new(burst_of(group, log, clock))),
    })
}

pub fn view(log: &Log, ask: &Ask<'_>) -> ActivityView {
    let clock = ask.clock;
    let frame = Frame::of(log, ask.now, ask.span);

    let mut buckets = vec![Bucket::default(); Pace::Buckets as usize];
    for (at, kind, _) in log.timeline.range((frame.start, Kind::Came, ProcessInstance::default())..) {
        let Some(bucket) = frame.bucket(*at) else {
            break;
        };
        match kind {
            Kind::Came => buckets[bucket].came += 1,
            Kind::Went => buckets[bucket].went += 1,
        }
    }

    let (from, to) = ask.range.unwrap_or((frame.start, frame.end()));
    let picked = ask.range.and_then(|(from, to)| {
        let first = frame.bucket(from)?;
        let last = frame.bucket(to.saturating_sub(1).max(from))?;
        Some((first, last + 1))
    });

    let window = |at: u64| (from..to).contains(&at);
    let span = (from, Kind::Came, ProcessInstance::default())..(to, Kind::Came, ProcessInstance::default());
    let (mut came_count, mut went_count) = (0, 0);
    for (_, kind, _) in log.timeline.range(span.clone()) {
        match kind {
            Kind::Came => came_count += 1,
            Kind::Went => went_count += 1,
        }
    }

    let shown_kind = |kind: &Kind| match kind {
        Kind::Came => ask.filter.came,
        Kind::Went => ask.filter.went,
    };
    let mut walk = Walk {
        log,
        filter: ask.filter,
        text: ask.filter.text.trim().to_lowercase(),
        rows: Vec::new(),
        open: HashMap::new(),
    };
    let mut unwalked = 0;
    let mut events = log.timeline.range(span).rev();
    while let Some((at, kind, instance)) = events.next() {
        walk.settle(*at);
        if walk.enough() {
            unwalked = usize::from(shown_kind(kind)) + events.filter(|(_, kind, _)| shown_kind(kind)).count();
            break;
        }
        match kind {
            Kind::Came => {
                if let Some(started) = log.started.get(instance) {
                    walk.came(*at, *instance, log.launched_by(started));
                }
            }
            Kind::Went => {
                let shown = ask.filter.came
                    && log.started.get(instance).is_some_and(|started| window(started.came.at));
                if !shown && log.ended.contains_key(instance) {
                    walk.keep(*at, Light::Went(*instance));
                }
            }
        }
    }
    let mut rows = walk.finish();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.key().cmp(&a.1.key())));
    let earlier = rows.len().saturating_sub(Pace::Shown) + unwalked;
    rows.truncate(Pace::Shown);

    let quarter = frame.width * Pace::Buckets / 4;
    ActivityView {
        histogram: Histogram {
            buckets,
            ticks: (0..5).map(|n| clock(frame.start + quarter * n)).collect(),
            picked,
        },
        rows: rows.into_iter().filter_map(|(_, row)| row_of(log, row, clock)).collect(),
        earlier,
        came: came_count,
        went: went_count,
        from: clock(from),
        to: clock(to),
        history_since: log.since.map(clock),
        lost: log.lost,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::activity::{ActivityRow, Launcher};
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

    fn look(log: &Log, filter: &Filter, range: Option<(u64, u64)>) -> ActivityView {
        view(
            log,
            &Ask {
                now: at(59, 59),
                span: Span::Hour,
                filter,
                range,
                clock: utc,
            },
        )
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

    #[test]
    fn quick_starts_from_one_launcher_fold_into_a_burst() {
        let mut log = Log::default();
        git_burst(&mut log, 5);

        let rows = rows(&log);
        let [ActivityRow::Burst(burst)] = rows.as_slice() else {
            panic!("one burst: {rows:#?}");
        };
        assert_eq!(&*burst.launcher, "Code.exe");
        assert_eq!(burst.members.len(), 10);
        assert_eq!(burst.went, 10);
        assert_eq!(
            burst.names,
            [(Arc::from("cmd.exe"), 5), (Arc::from("git.exe"), 5)]
        );
    }

    #[test]
    fn two_starts_are_not_a_burst() {
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
    fn hiding_bursts_hides_their_members_too() {
        let mut log = Log::default();
        git_burst(&mut log, 5);
        log.record(&[came(20, 1, "tool.exe", at(30, 0))]);
        let filter = Filter {
            bursts: false,
            ..Filter::default()
        };

        let rows = look(&log, &filter, None).rows;
        assert_eq!(rows.len(), 1, "{rows:#?}");
        assert_eq!(&*only_came(&rows[0]).name, "tool.exe");
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
            came(21, 1, "setup.exe", at(11, 0)),
            came(22, 1, "setup.exe", at(12, 0)),
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

    #[test]
    fn the_histogram_counts_each_minute_of_the_hour() {
        let mut log = Log::default();
        log.note_running(&[running(30, "old.exe", at(0, 0))]);
        log.record(&[
            came(20, 1, "a.exe", at(10, 5)),
            came(21, 1, "b.exe", at(10, 50)),
            went(30, at(42, 0)),
        ]);

        let histogram = look(&log, &Filter::default(), None).histogram;
        assert_eq!(histogram.buckets.len(), 60);
        assert_eq!(histogram.buckets[10].came, 2);
        assert_eq!(histogram.buckets[42].went, 1);
        assert_eq!(histogram.buckets.iter().map(|b| b.came + b.went).sum::<u32>(), 3);
    }

    #[test]
    fn a_picked_bucket_narrows_the_rows_to_its_minute() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 5)), came(21, 1, "b.exe", at(11, 5))]);

        let range = bucket_range(&log, at(59, 59), Span::Hour, 10).expect("a bucket of the hour");
        let view = look(&log, &Filter::default(), Some(range));
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(&*only_came(&view.rows[0]).name, "a.exe");
        assert_eq!(view.histogram.picked, Some((10, 11)));
        assert_eq!((view.came, view.went), (1, 0));
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
