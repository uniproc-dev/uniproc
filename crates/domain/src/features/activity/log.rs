use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use app_contracts::features::activity::{
    ActivityRow, ActivityView, Area, Came, Clock, Dot, Exit, Filter, Group, Hue, Launcher, Legend, Lived, Pick,
    Piece, Run, Scatter, Series, Span, Went,
};
use std::hash::BuildHasher;
use std::num::NonZeroU32;

use hashbrown::hash_table::Entry;
use hashbrown::{DefaultHashBuilder, HashTable};

use app_contracts::features::agents::{
    ProcessEvent, ProcessInstance, ScheduledTask, WindowsProcessEvents, WindowsProcessStats,
};

use super::names::{id, index, Folder, Names, Spell, Text, Texts, Word};
use super::packed::{Numbers, Pool};

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

struct Flag;

#[expect(non_upper_case_globals)]
impl Flag {
    const FirstSeen: u8 = 1;
    const ElevationKnown: u8 = 2;
    const Elevated: u8 = 4;
}

struct Started {
    at: u64,
    sequence: u64,
    parent_sequence: u64,
    launcher_sequence: u64,
    went: u64,
    pid: u32,
    parent_pid: u32,
    launcher_pid: u32,
    name: Spell,
    folder: Option<Folder>,
    launcher: Option<Spell>,
    path: Option<Text>,
    command: Option<Text>,
    dir: Option<Text>,
    user: Option<Text>,
    task: Option<NonZeroU32>,
    services: Option<NonZeroU32>,
    session: u32,
    flags: u8,
    graded: Cell<Graded>,
}

impl Started {
    fn instance(&self) -> ProcessInstance {
        ProcessInstance {
            pid: self.pid,
            sequence: self.sequence,
        }
    }

    fn parent(&self) -> ProcessInstance {
        ProcessInstance {
            pid: self.parent_pid,
            sequence: self.parent_sequence,
        }
    }

    fn launched(&self) -> Option<(ProcessInstance, Spell)> {
        let launcher = ProcessInstance {
            pid: self.launcher_pid,
            sequence: self.launcher_sequence,
        };
        self.launcher.map(|name| (launcher, name))
    }

    fn went_at(&self) -> Option<u64> {
        (self.went != Never).then_some(self.went)
    }

    fn first_seen(&self) -> bool {
        self.flags & Flag::FirstSeen != 0
    }

    fn elevated(&self) -> Option<bool> {
        (self.flags & Flag::ElevationKnown != 0).then_some(self.flags & Flag::Elevated != 0)
    }
}

#[expect(non_upper_case_globals)]
const Never: u64 = u64::MAX;

struct Ended {
    at: u64,
    sequence: u64,
    started_at: u64,
    came: u64,
    pid: u32,
    file: Option<Spell>,
    folder: Option<Folder>,
    image_name: Option<Spell>,
    numbers: u32,
    graded: Cell<Graded>,
}

impl Ended {
    fn instance(&self) -> ProcessInstance {
        ProcessInstance {
            pid: self.pid,
            sequence: self.sequence,
        }
    }

    fn came_at(&self) -> Option<u64> {
        (self.came != Never).then_some(self.came)
    }

    fn started_at(&self) -> Option<u64> {
        (self.started_at != Never).then_some(self.started_at)
    }
}

#[derive(Clone, Copy, Default)]
struct Graded {
    asked: u32,
    named: u32,
    verdict: u16,
}

impl Graded {
    fn verdict(self) -> Option<Option<usize>> {
        match self.verdict {
            0 => None,
            1 => Some(None),
            group => Some(Some(usize::from(group) - 2)),
        }
    }

    fn of(verdict: Option<Option<usize>>) -> u16 {
        match verdict {
            None => 0,
            Some(None) => 1,
            Some(Some(group)) => u16::try_from(group + 2).unwrap_or(u16::MAX),
        }
    }
}

#[derive(Clone, Copy)]
struct Slot {
    at: u64,
    event: u32,
}

#[expect(non_upper_case_globals)]
impl Slot {
    const Went: u32 = 1 << 31;

    fn came(at: u64, index: usize) -> Self {
        Self { at, event: index as u32 }
    }

    fn went(at: u64, index: usize) -> Self {
        Self {
            at,
            event: index as u32 | Self::Went,
        }
    }

    fn index(self) -> usize {
        (self.event & !Self::Went) as usize
    }

    fn is_went(self) -> bool {
        self.event & Self::Went != 0
    }
}

enum Event<'a> {
    Came(&'a Started),
    Went(&'a Ended),
}

impl Event<'_> {
    fn at(&self) -> u64 {
        match self {
            Event::Came(started) => started.at,
            Event::Went(ended) => ended.at,
        }
    }
}

fn event_of<'a>(starts: &'a [Started], ends: &'a [Ended], slot: Slot) -> Event<'a> {
    if slot.is_went() {
        Event::Went(&ends[slot.index()])
    } else {
        Event::Came(&starts[slot.index()])
    }
}

fn key_of(starts: &[Started], ends: &[Ended], slot: Slot) -> (u64, Kind, ProcessInstance) {
    match event_of(starts, ends, slot) {
        Event::Came(started) => (slot.at, Kind::Came, started.instance()),
        Event::Went(ended) => (slot.at, Kind::Went, ended.instance()),
    }
}

#[derive(Clone, Copy, Default)]
struct Slots {
    came: Option<NonZeroU32>,
    went: Option<NonZeroU32>,
}

fn instance_in(starts: &[Started], ends: &[Ended], slots: Slots) -> ProcessInstance {
    match (slots.came, slots.went) {
        (Some(came), _) => starts[index(came)].instance(),
        (None, Some(went)) => ends[index(went)].instance(),
        (None, None) => ProcessInstance::default(),
    }
}

struct Known {
    name: Spell,
    start_time: u64,
}

#[derive(Clone, Copy)]
enum Rule {
    Exe(Option<Word>),
    Folder(Option<Folder>),
    Under(Option<Word>),
}

pub struct Log {
    names: Names,
    shells: [Word; Pace::Shells.len()],
    console_host: Word,
    texts: Texts,
    numbers: Numbers,
    tasks: Vec<ScheduledTask>,
    task_ids: HashMap<(Arc<str>, Arc<str>), NonZeroU32>,
    service_lists: Vec<Arc<[Arc<str>]>>,
    service_ids: HashMap<Arc<[Arc<str>]>, NonZeroU32>,
    timeline: VecDeque<Slot>,
    starts: Vec<Started>,
    ends: Vec<Ended>,
    index: HashTable<Slots>,
    hasher: DefaultHashBuilder,
    seen: HashSet<Option<Folder>>,
    habits: HashMap<(Word, Option<Folder>), usize>,
    unplaced: Vec<ProcessInstance>,
    running: HashMap<u32, Vec<Known>>,
    since: Option<u64>,
    lost: u64,
    asked: RefCell<(Filter, Vec<Group>)>,
    asking: Cell<u32>,
    naming: Vec<u64>,
    shown: RefCell<HashMap<ProcessInstance, Shown>>,
    touched: HashMap<u64, u32>,
    drawn: RefCell<HashMap<u64, Drawn>>,
    spare: RefCell<Pool<Dot>>,
}

impl Default for Log {
    fn default() -> Self {
        let mut names = Names::default();
        let shells = Pace::Shells.map(|shell| names.word(shell));
        let console_host = names.word(Pace::ConsoleHost);
        Self {
            names,
            shells,
            console_host,
            texts: Texts::default(),
            numbers: Numbers::default(),
            tasks: Vec::new(),
            task_ids: HashMap::new(),
            service_lists: Vec::new(),
            service_ids: HashMap::new(),
            timeline: VecDeque::new(),
            starts: Vec::new(),
            ends: Vec::new(),
            index: HashTable::new(),
            hasher: DefaultHashBuilder::default(),
            seen: HashSet::new(),
            habits: HashMap::new(),
            unplaced: Vec::new(),
            running: HashMap::new(),
            since: None,
            lost: 0,
            asked: RefCell::new((Filter::default(), Vec::new())),
            asking: Cell::new(1),
            naming: Vec::new(),
            shown: RefCell::default(),
            touched: HashMap::new(),
            drawn: RefCell::default(),
            spare: RefCell::default(),
        }
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

fn admitted(only: Option<Rule>, hidden: &[Rule], has: impl Fn(Rule) -> bool) -> bool {
    only.is_none_or(&has) && !hidden.iter().copied().any(has)
}

struct Groups<'a> {
    all: &'a [Group],
    rules: Vec<Vec<Rule>>,
}

impl<'a> Groups<'a> {
    fn new(log: &Log, all: &'a [Group]) -> Self {
        Self {
            all,
            rules: all
                .iter()
                .map(|group| group.rules.iter().map(|pick| log.rule(pick)).collect())
                .collect(),
        }
    }

    fn of(&self, has: impl Fn(Rule) -> bool) -> Option<usize> {
        self.rules.iter().position(|rules| rules.iter().copied().any(&has))
    }

    fn hue(&self, group: Option<usize>) -> Option<Hue> {
        group.map(|group| self.all[group].hue)
    }
}

impl Log {
    pub fn record(&mut self, events: &[ProcessEvent]) {
        for event in events {
            self.add(event, Self::insert);
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
        for event in events {
            self.add(event, Self::append);
        }
        let (starts, ends) = (&self.starts, &self.ends);
        self.timeline
            .make_contiguous()
            .sort_by_key(|&slot| key_of(starts, ends, slot));
        self.regrade();
    }

    fn regrade(&self) {
        self.asking.set(self.asking.get() + 1);
    }

    fn renamed(&mut self, from: u64) {
        for floor in self.naming.iter_mut().rev() {
            if *floor <= from {
                break;
            }
            *floor = from;
        }
        self.naming.push(from);
    }

    fn named_since(&self, named: u32, at: u64) -> bool {
        self.naming.get(named as usize).is_none_or(|&from| from > at)
    }

    fn touch(&mut self, at: u64) {
        let touched = self.touched.entry(at / Ticks::Minute).or_default();
        *touched = touched.wrapping_add(1);
    }

    fn touched(&self, minute: u64) -> u32 {
        self.touched.get(&minute).copied().unwrap_or_default()
    }

    fn add(&mut self, event: &ProcessEvent, place: fn(&mut Self, Slot)) {
        match event {
            ProcessEvent::Came(came) => {
                if self.slots(came.instance).came.is_some() {
                    return;
                }
                let placed = self.names.path(&came.image_path);
                let first_seen = came.image_path.is_empty() || self.seen.insert(placed.folder);
                let name = match placed.file {
                    Some(file) => file,
                    None => self
                        .names
                        .spell(came.command_line.split_whitespace().next().map(file_name).unwrap_or_default()),
                };
                let went = self.ended_mut(came.instance).map_or(Never, |ended| {
                    ended.came = came.at;
                    ended.graded.take();
                    ended.at
                });
                self.touch(came.at);
                if went != Never {
                    self.touch(went);
                }
                let mut flags = 0;
                if first_seen {
                    flags |= Flag::FirstSeen;
                }
                if let Some(elevated) = came.elevated {
                    flags |= Flag::ElevationKnown;
                    if elevated {
                        flags |= Flag::Elevated;
                    }
                }
                let started = Started {
                    at: came.at,
                    sequence: came.instance.sequence,
                    parent_sequence: came.parent.sequence,
                    launcher_sequence: 0,
                    went,
                    pid: came.instance.pid,
                    parent_pid: came.parent.pid,
                    launcher_pid: 0,
                    name,
                    folder: placed.folder,
                    launcher: None,
                    path: self.keep(&came.image_path),
                    command: self.keep(&came.command_line),
                    dir: self.keep(&came.working_dir),
                    user: self.keep(&came.user),
                    task: came.scheduled_task.as_ref().map(|task| self.task(task)),
                    services: (!came.parent_services.is_empty()).then(|| self.services(&came.parent_services)),
                    session: came.session_id,
                    flags,
                    graded: Cell::default(),
                };
                let at = self.starts.len();
                self.starts.push(started);
                self.link(came.instance, |slots| slots.came = Some(id(at)));
                place(self, Slot::came(came.at, at));
                if !self.settle_launcher(came.instance) {
                    self.unplaced.push(came.instance);
                }
            }
            ProcessEvent::Went(went) => {
                if self.slots(went.instance).went.is_some() {
                    return;
                }
                let placed = self.names.path(&went.image_path);
                let came = self.started_mut(went.instance).map_or(Never, |started| {
                    started.went = went.at;
                    started.at
                });
                self.touch(went.at);
                if came != Never {
                    self.touch(came);
                }
                let image_name = (!went.image_name.is_empty()).then(|| self.names.spell(&went.image_name));
                let numbers = self.numbers.put([
                    u64::from(went.exit_code),
                    went.cpu_cycles,
                    went.io_read_bytes,
                    went.io_write_bytes,
                    went.peak_commit_bytes,
                ]);
                let at = self.ends.len();
                self.ends.push(Ended {
                    at: went.at,
                    sequence: went.instance.sequence,
                    started_at: went.started_at.unwrap_or(Never),
                    came,
                    pid: went.instance.pid,
                    file: placed.file,
                    folder: placed.folder,
                    image_name,
                    numbers,
                    graded: Cell::default(),
                });
                self.link(went.instance, |slots| slots.went = Some(id(at)));
                place(self, Slot::went(went.at, at));
            }
        }
    }

    fn keep(&mut self, text: &str) -> Option<Text> {
        (!text.is_empty()).then(|| self.texts.put(text))
    }

    fn task(&mut self, task: &ScheduledTask) -> NonZeroU32 {
        let key = (task.name.clone(), task.path.clone());
        if let Some(&known) = self.task_ids.get(&key) {
            return known;
        }
        let new = id(self.tasks.len());
        self.tasks.push(task.clone());
        self.task_ids.insert(key, new);
        new
    }

    fn services(&mut self, services: &Arc<[Arc<str>]>) -> NonZeroU32 {
        if let Some(&known) = self.service_ids.get(services) {
            return known;
        }
        let new = id(self.service_lists.len());
        self.service_lists.push(services.clone());
        self.service_ids.insert(services.clone(), new);
        new
    }

    fn kept(&self, text: Option<Text>) -> &str {
        text.map_or("", |text| self.texts.get(text))
    }

    fn exit(&self, ended: &Ended, since: Option<u64>, clock: fn(u64) -> Clock) -> Exit {
        let [code, cpu_cycles, read_bytes, write_bytes, peak_commit_bytes] = self.numbers.get(ended.numbers);
        Exit {
            at: clock(ended.at),
            code: code as u32,
            lived: since.map_or(0, |since| ended.at.saturating_sub(since)),
            cpu_cycles,
            read_bytes,
            write_bytes,
            peak_commit_bytes,
        }
    }

    fn insert(&mut self, slot: Slot) {
        let (starts, ends) = (&self.starts, &self.ends);
        let key = key_of(starts, ends, slot);
        let last = self.timeline.back().map(|&last| key_of(starts, ends, last));
        if last.is_none_or(|last| last <= key) {
            self.timeline.push_back(slot);
            return;
        }
        let position = self.timeline.partition_point(|&known| key_of(starts, ends, known) < key);
        self.timeline.insert(position, slot);
        if !slot.is_went() {
            self.regrade();
        }
    }

    fn append(&mut self, slot: Slot) {
        self.timeline.push_back(slot);
    }

    fn slots(&self, instance: ProcessInstance) -> Slots {
        let (starts, ends) = (&self.starts, &self.ends);
        self.index
            .find(self.hasher.hash_one(instance), |&slots| instance_in(starts, ends, slots) == instance)
            .copied()
            .unwrap_or_default()
    }

    fn link(&mut self, instance: ProcessInstance, change: impl FnOnce(&mut Slots)) {
        let (starts, ends, hasher) = (&self.starts, &self.ends, &self.hasher);
        let entry = self.index.entry(
            hasher.hash_one(instance),
            |&slots| instance_in(starts, ends, slots) == instance,
            |&slots| hasher.hash_one(instance_in(starts, ends, slots)),
        );
        match entry {
            Entry::Occupied(mut occupied) => change(occupied.get_mut()),
            Entry::Vacant(vacant) => {
                let mut slots = Slots::default();
                change(&mut slots);
                vacant.insert(slots);
            }
        }
    }

    fn started(&self, instance: ProcessInstance) -> Option<&Started> {
        self.starts.get(index(self.slots(instance).came?))
    }

    fn ended(&self, instance: ProcessInstance) -> Option<&Ended> {
        self.ends.get(index(self.slots(instance).went?))
    }

    fn started_mut(&mut self, instance: ProcessInstance) -> Option<&mut Started> {
        let came = self.slots(instance).came?;
        self.starts.get_mut(index(came))
    }

    fn ended_mut(&mut self, instance: ProcessInstance) -> Option<&mut Ended> {
        let went = self.slots(instance).went?;
        self.ends.get_mut(index(went))
    }

    fn span(&self, from: u64, to: u64) -> impl DoubleEndedIterator<Item = Event<'_>> {
        let start = self.timeline.partition_point(|slot| slot.at < from);
        let end = self.timeline.partition_point(|slot| slot.at < to);
        self.timeline
            .range(start..end.max(start))
            .map(|&slot| event_of(&self.starts, &self.ends, slot))
    }

    pub fn note_running(&mut self, processes: &[WindowsProcessStats]) {
        let mut renamed = None::<u64>;
        for process in processes {
            if !process.image_path.is_empty() {
                let placed = self.names.path(&process.image_path);
                self.seen.insert(placed.folder);
            }
            let owners = self.running.entry(process.pid).or_default();
            if owners.last().is_none_or(|known| known.start_time != process.start_time) {
                let name = self.names.spell(&process.name);
                owners.push(Known {
                    name,
                    start_time: process.start_time,
                });
                renamed = Some(renamed.map_or(process.start_time, |from| from.min(process.start_time)));
            }
        }
        if let Some(from) = renamed {
            self.renamed(from);
        }
        for instance in std::mem::take(&mut self.unplaced) {
            self.settle_launcher(instance);
        }
    }

    fn settle_launcher(&mut self, instance: ProcessInstance) -> bool {
        let Some(started) = self.started(instance) else {
            return true;
        };
        let (folder, at) = (started.folder, started.at);
        let Some(launcher) = self.launched_by(started) else {
            return false;
        };
        let habit = self.habits.entry((self.names.word_of(launcher.1), folder)).or_default();
        *habit += 1;
        if *habit == Pace::Routine {
            self.regrade();
        }
        if let Some(started) = self.started_mut(instance) {
            let (by, name) = launcher;
            started.launcher_pid = by.pid;
            started.launcher_sequence = by.sequence;
            started.launcher = Some(name);
        }
        self.touch(at);
        true
    }

    fn ask(&self, filter: &Filter, groups: &[Group]) -> u32 {
        let mut asked = self.asked.borrow_mut();
        if asked.0 != *filter || asked.1 != groups {
            *asked = (filter.clone(), groups.to_vec());
            self.asking.set(self.asking.get() + 1);
        }
        self.asking.get()
    }

    fn known(&self, pid: u32, before: u64) -> Option<&Known> {
        self.running
            .get(&pid)?
            .iter()
            .rev()
            .find(|known| known.start_time <= before)
    }

    fn rule(&self, pick: &Pick) -> Rule {
        match pick {
            Pick::Exe(file) => Rule::Exe(self.names.find_word(file)),
            Pick::Folder(folder) => Rule::Folder(self.names.find_folder(folder)),
            Pick::Under(name) => Rule::Under(self.names.find_word(name)),
        }
    }

    fn picks(&self, started: &Started) -> Vec<Pick> {
        let mut picks = Vec::new();
        if !self.names.text(started.name).is_empty() {
            picks.push(Pick::Exe(self.names.folded(self.names.word_of(started.name)).clone()));
        }
        if let Some(folder) = started.folder {
            picks.push(Pick::Folder(self.names.folder_text(folder)));
        }
        if let Some(launcher) = started.launcher {
            picks.push(Pick::Under(self.names.folded(self.names.word_of(launcher)).clone()));
        }
        picks
    }

    fn is_shell(&self, name: Spell) -> bool {
        self.shells.contains(&self.names.word_of(name))
    }

    fn text(&self, name: Spell) -> Arc<str> {
        self.names.text(name).clone()
    }

    fn routine(&self, started: &Started) -> bool {
        started.launcher.is_some_and(|launcher| {
            self.habits
                .get(&(self.names.word_of(launcher), started.folder))
                .is_some_and(|&count| count >= Pace::Routine)
        })
    }

    fn hosted(&self, started: &Started) -> bool {
        self.names.word_of(started.name) == self.console_host
            && self.named(started.parent(), started.at).is_some()
    }

    fn has(&self, started: &Started, rule: Rule) -> bool {
        match rule {
            Rule::Exe(file) => file == Some(self.names.word_of(started.name)),
            Rule::Folder(folder) => folder.is_some() && folder == started.folder,
            Rule::Under(name) => name.is_some_and(|name| self.under(started, name)),
        }
    }

    fn under(&self, started: &Started, wanted: Word) -> bool {
        let mut next = Some((started.parent(), started.at));
        for _ in 0..Pace::ChainDepth {
            let Some((at, before)) = next else {
                return false;
            };
            if self.named(at, before).is_some_and(|name| self.names.word_of(name) == wanted) {
                return true;
            }
            next = self.started(at).map(|started| (started.parent(), started.at));
        }
        false
    }

    fn went_has(&self, ended: &Ended, rule: Rule) -> bool {
        if let Some(started) = self.started(ended.instance()) {
            return self.has(started, rule);
        }
        match rule {
            Rule::Exe(file) => file.is_some() && file == ended.file.map(|file| self.names.word_of(file)),
            Rule::Folder(folder) => folder.is_some() && folder == ended.folder,
            Rule::Under(_) => false,
        }
    }

    fn came_group(&self, groups: &Groups<'_>, started: &Started) -> Option<usize> {
        groups.of(|rule| self.has(started, rule))
    }

    fn went_group(&self, groups: &Groups<'_>, ended: &Ended) -> Option<usize> {
        groups.of(|rule| self.went_has(ended, rule))
    }

    fn began(&self) -> Option<u64> {
        let first = self.timeline.front().map(|slot| slot.at);
        match (self.since, first) {
            (Some(since), Some(first)) => Some(since.min(first)),
            (since, first) => since.or(first),
        }
    }

    fn named(&self, instance: ProcessInstance, before: u64) -> Option<Spell> {
        self.started(instance)
            .map(|started| started.name)
            .or_else(|| self.known(instance.pid, before).map(|known| known.name))
    }

    fn ancestors<'a>(&'a self, started: &Started) -> impl Iterator<Item = (ProcessInstance, Spell)> + 'a {
        let mut next = Some((started.parent(), started.at));
        std::iter::from_fn(move || {
            let (at, before) = next.take()?;
            let name = self.named(at, before)?;
            if self.is_shell(name) {
                next = self.started(at).map(|parent| (parent.parent(), parent.at));
            }
            Some((at, name))
        })
        .take(Pace::ChainDepth)
    }

    fn launched_by(&self, started: &Started) -> Option<(ProcessInstance, Spell)> {
        self.ancestors(started).last().filter(|&(_, name)| !self.is_shell(name))
    }

    fn lineage(&self, started: &Started) -> (Vec<Arc<str>>, Option<(ProcessInstance, Spell)>) {
        let mut chain = vec![self.text(started.name)];
        let mut launched_by = None;
        for (at, name) in self.ancestors(started) {
            if !self.is_shell(name) {
                launched_by = Some((at, name));
            }
            chain.push(self.text(name));
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

fn came_lived(started: &Started) -> Lived {
    started
        .went_at()
        .map_or(Lived::Running, |at| Lived::For(at.saturating_sub(started.at)))
}

fn went_lived(ended: &Ended) -> Lived {
    ended
        .came_at()
        .map_or(Lived::Unknown, |at| Lived::For(ended.at.saturating_sub(at)))
}

struct Drawn {
    asked: u32,
    named: u32,
    touched: u32,
    dots: Arc<Vec<Dot>>,
    runs: Arc<[Run]>,
    legend: Legend,
}

impl Drawn {
    fn holds(&self, sieve: &Sieve<'_>, minute: u64) -> bool {
        let log = sieve.log;
        self.asked == sieve.asked
            && log.named_since(self.named, (minute + 1) * Ticks::Minute - 1)
            && self.touched == log.touched(minute)
    }
}

fn plotted(sieve: &Sieve<'_>, event: &Event<'_>) -> Option<(Option<usize>, Dot)> {
    let log = sieve.log;
    let (instance, graded, lived, faint) = match *event {
        Event::Came(started) => {
            if log.hosted(started) {
                return None;
            }
            let instance = started.instance();
            let graded = sieve
                .grade_came(started)
                .or_else(|| log.ended(instance).and_then(|ended| sieve.grade_went(ended)));
            (instance, graded, came_lived(started), log.routine(started))
        }
        Event::Went(ended) => {
            if ended.came_at().is_some() {
                return None;
            }
            (ended.instance(), sieve.grade_went(ended), Lived::Unknown, false)
        }
    };
    let group = graded?;
    Some((group, Dot::new(instance, event.at(), lived, faint, sieve.hue(group))))
}

fn counted(legend: &mut Legend, group: Option<usize>) {
    match group {
        Some(group) => legend.groups[group] += 1,
        None => legend.other += 1,
    }
}

fn draw(sieve: &Sieve<'_>, minute: u64, mut dots: Arc<Vec<Dot>>) -> Drawn {
    let log = sieve.log;
    let mut legend = Legend {
        groups: vec![0; sieve.groups.all.len()],
        other: 0,
    };
    let kept = Arc::make_mut(&mut dots);
    for event in log.span(minute * Ticks::Minute, (minute + 1) * Ticks::Minute) {
        if let Some((group, dot)) = plotted(sieve, &event) {
            counted(&mut legend, group);
            if sieve.shown(group) {
                kept.push(dot);
            }
        }
    }
    kept.sort_by_key(Dot::look);
    let runs = Run::split(kept);
    Drawn {
        asked: sieve.asked,
        named: log.naming.len() as u32,
        touched: log.touched(minute),
        dots,
        runs: runs.into(),
        legend,
    }
}

fn scatter(sieve: &Sieve<'_>, frame: &Frame, clock: fn(u64) -> Clock) -> (Scatter, Legend) {
    let log = sieve.log;
    let mut legend = Legend {
        groups: vec![0; sieve.groups.all.len()],
        other: 0,
    };
    let (first, last) = (frame.start / Ticks::Minute, (frame.end - 1) / Ticks::Minute);
    let mut drawn = log.drawn.borrow_mut();
    let mut spare = log.spare.borrow_mut();
    for (_, gone) in drawn.extract_if(|&minute, _| !(first..=last).contains(&minute)) {
        spare.retire(gone.dots);
    }
    let mut pieces = Vec::with_capacity((last - first + 1) as usize);
    for minute in first..=last {
        let kept = match drawn.remove(&minute) {
            Some(kept) if kept.holds(sieve, minute) => kept,
            stale => {
                if let Some(stale) = stale {
                    spare.retire(stale.dots);
                }
                draw(sieve, minute, spare.take())
            }
        };
        let (start, end) = (minute * Ticks::Minute, (minute + 1) * Ticks::Minute);
        let runs = if frame.start <= start && end <= frame.end {
            for (total, count) in legend.groups.iter_mut().zip(&kept.legend.groups) {
                *total += count;
            }
            legend.other += kept.legend.other;
            kept.runs.clone()
        } else {
            for event in log.span(start.max(frame.start), end.min(frame.end)) {
                if let Some((group, _)) = plotted(sieve, &event) {
                    counted(&mut legend, group);
                }
            }
            kept.runs
                .iter()
                .filter_map(|run| {
                    let dots = &kept.dots[run.from..run.to];
                    let from = run.from + dots.partition_point(|dot| dot.at() < frame.start);
                    let to = run.from + dots.partition_point(|dot| dot.at() < frame.end);
                    (from < to).then_some(Run { from, to, ..*run })
                })
                .collect()
        };
        if !runs.is_empty() {
            pieces.push(Piece {
                dots: kept.dots.clone(),
                runs,
            });
        }
        drawn.insert(minute, kept);
    }
    let scatter = Scatter {
        pieces,
        now: frame.end - 1,
        now_clock: clock(frame.end - 1),
        length: frame.length(),
        area: None,
    };
    (scatter, legend)
}

fn came_of(log: &Log, groups: &Groups<'_>, started: &Started, clock: fn(u64) -> Clock) -> Came {
    let (chain, launched_by) = log.lineage(started);
    let services: Arc<[Arc<str>]> = started
        .services
        .map_or_else(|| Arc::from([]), |services| log.service_lists[index(services)].clone());
    let by_parent = launched_by.is_some_and(|(at, _)| at == started.parent()) && !services.is_empty();
    let launcher = match (started.task, launched_by) {
        (Some(task), _) => Launcher::Task(log.tasks[index(task)].clone()),
        (None, Some(_)) if by_parent => Launcher::Services(services.clone()),
        (None, Some((_, name))) => Launcher::Process(log.text(name)),
        (None, None) => Launcher::Unknown,
    };
    Came {
        key: started.instance(),
        at: clock(started.at),
        name: log.text(started.name),
        image_path: log.kept(started.path).into(),
        command_line: log.kept(started.command).into(),
        working_dir: log.kept(started.dir).into(),
        user: log.kept(started.user).into(),
        session_id: started.session,
        elevated: started.elevated(),
        launcher,
        chain,
        parent_services: services,
        first_seen: started.first_seen(),
        exit: log
            .ended(started.instance())
            .map(|ended| log.exit(ended, Some(started.at), clock)),
        picks: log.picks(started),
        hue: groups.hue(log.came_group(groups, started)),
    }
}

fn went_name(log: &Log, ended: &Ended) -> Option<Arc<str>> {
    log.started(ended.instance())
        .map(|started| log.text(started.name))
        .or_else(|| ended.file.map(|file| log.text(file)))
        .or_else(|| ended.image_name.map(|name| log.text(name)))
        .or_else(|| log.named(ended.instance(), ended.at).map(|name| log.text(name)))
}

fn went_of(log: &Log, groups: &Groups<'_>, ended: &Ended, clock: fn(u64) -> Clock) -> Went {
    let started = ended.came_at().or(ended.started_at()).or_else(|| {
        log.known(ended.pid, ended.at)
            .map(|known| known.start_time)
            .filter(|&start| start > 0)
    });
    Went {
        key: ended.instance(),
        name: went_name(log, ended),
        lived: started.map(|start| ended.at - start),
        exit: log.exit(ended, started, clock),
        hue: groups.hue(log.went_group(groups, ended)),
    }
}

struct Gathered<'a> {
    members: Vec<&'a Started>,
    oldest: &'a Started,
    count: usize,
    went: usize,
    names: HashMap<Spell, usize>,
    file: Word,
    one_file: bool,
}

impl<'a> Gathered<'a> {
    fn new(log: &Log, started: &'a Started) -> Self {
        Self {
            members: vec![started],
            oldest: started,
            count: 1,
            went: usize::from(started.went_at().is_some()),
            names: HashMap::from([(started.name, 1)]),
            file: log.names.word_of(started.name),
            one_file: true,
        }
    }

    fn add(&mut self, log: &Log, started: &'a Started) {
        if self.members.len() < Pace::Shown {
            self.members.push(started);
        }
        self.oldest = started;
        self.count += 1;
        self.went += usize::from(started.went_at().is_some());
        *self.names.entry(started.name).or_default() += 1;
        self.one_file &= log.names.word_of(started.name) == self.file;
    }
}

enum Light<'a> {
    Came(ProcessInstance),
    Went(ProcessInstance),
    Series(Gathered<'a>),
}

impl Light<'_> {
    fn key(&self) -> ProcessInstance {
        match self {
            Light::Came(instance) | Light::Went(instance) => *instance,
            Light::Series(gathered) => gathered.oldest.instance(),
        }
    }
}

struct Shown {
    asked: u32,
    named: u32,
    went_at: Option<u64>,
    launched: Option<(ProcessInstance, Spell)>,
    row: Arc<Came>,
}

struct Rows<'a> {
    log: &'a Log,
    groups: &'a Groups<'a>,
    clock: fn(u64) -> Clock,
    asked: u32,
    kept: HashMap<ProcessInstance, Shown>,
    shown: HashMap<ProcessInstance, Shown>,
}

impl<'a> Rows<'a> {
    fn new(log: &'a Log, groups: &'a Groups<'a>, clock: fn(u64) -> Clock, asked: u32) -> Self {
        Self {
            log,
            groups,
            clock,
            asked,
            kept: HashMap::new(),
            shown: HashMap::new(),
        }
    }

    fn came(&mut self, started: &Started) -> Arc<Came> {
        let instance = started.instance();
        if let Some(shown) = self.shown.get(&instance) {
            return shown.row.clone();
        }
        let kept = self.kept.remove(&instance).filter(|kept| {
            kept.asked == self.asked
                && kept.went_at == started.went_at()
                && kept.launched == started.launched()
                && self.log.named_since(kept.named, started.at)
        });
        let shown = kept.unwrap_or_else(|| Shown {
            asked: self.asked,
            named: self.log.naming.len() as u32,
            went_at: started.went_at(),
            launched: started.launched(),
            row: Arc::new(came_of(self.log, self.groups, started, self.clock)),
        });
        let row = shown.row.clone();
        self.shown.insert(instance, shown);
        row
    }

    fn series(&mut self, gathered: &Gathered<'_>) -> Option<Series> {
        let log = self.log;
        let newest = *gathered.members.first()?;
        let launcher = newest.launcher?;
        let folder = log.names.folder_text(newest.folder?);
        let mut names: Vec<_> = gathered.names.iter().map(|(&name, &count)| (log.text(name), count)).collect();
        names.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut picks = Vec::new();
        if gathered.one_file {
            picks.push(Pick::Exe(log.names.folded(gathered.file).clone()));
        }
        picks.push(Pick::Folder(folder.clone()));
        picks.push(Pick::Under(log.names.folded(log.names.word_of(launcher)).clone()));
        Some(Series {
            key: gathered.oldest.instance(),
            at: (self.clock)(newest.at),
            launcher: log.text(launcher),
            folder,
            names,
            count: gathered.count,
            went: gathered.went,
            routine: log.routine(newest),
            members: gathered.members.iter().map(|member| self.came(member)).collect(),
            picks,
            hue: self.groups.hue(log.came_group(self.groups, newest)),
        })
    }

    fn row(&mut self, row: Light<'_>) -> Option<ActivityRow> {
        let log = self.log;
        Some(match row {
            Light::Came(instance) => ActivityRow::Came(self.came(log.started(instance)?)),
            Light::Went(instance) => {
                ActivityRow::Went(Arc::new(went_of(log, self.groups, log.ended(instance)?, self.clock)))
            }
            Light::Series(gathered) => ActivityRow::Series(Arc::new(self.series(&gathered)?)),
        })
    }
}

struct Sieve<'a> {
    log: &'a Log,
    filter: &'a Filter,
    groups: &'a Groups<'a>,
    only: Option<Rule>,
    hidden: Vec<Rule>,
    text: String,
    band: Option<(Lived, Lived)>,
    asked: u32,
}

impl<'a> Sieve<'a> {
    fn new(log: &'a Log, filter: &'a Filter, groups: &'a Groups<'a>, area: Option<Area>, asked: u32) -> Self {
        Self {
            log,
            filter,
            groups,
            only: filter.only.as_ref().map(|pick| log.rule(pick)),
            hidden: filter.hidden.iter().map(|pick| log.rule(pick)).collect(),
            text: filter.text.trim().to_lowercase(),
            band: area.map(|area| (area.shortest, area.longest)),
            asked,
        }
    }

    fn cached(
        &self,
        graded: &Cell<Graded>,
        at: u64,
        judge: impl FnOnce() -> Option<Option<usize>>,
    ) -> Option<Option<usize>> {
        let known = graded.get();
        if known.asked == self.asked && self.log.named_since(known.named, at) {
            return known.verdict();
        }
        let verdict = judge();
        graded.set(Graded {
            asked: self.asked,
            named: self.log.naming.len() as u32,
            verdict: Graded::of(verdict),
        });
        verdict
    }

    fn held(&self, lived: Lived) -> bool {
        self.band.is_none_or(|(shortest, longest)| (shortest..=longest).contains(&lived))
    }

    fn shown(&self, group: Option<usize>) -> bool {
        group.map_or(self.filter.other, |group| self.groups.all[group].shown)
    }

    fn grade_came(&self, started: &Started) -> Option<Option<usize>> {
        self.cached(&started.graded, started.at, || self.judge_came(started))
            .filter(|_| self.held(came_lived(started)))
    }

    fn grade_went(&self, ended: &Ended) -> Option<Option<usize>> {
        self.cached(&ended.graded, ended.at, || self.judge_went(ended))
            .filter(|_| self.held(went_lived(ended)))
    }

    fn judge_came(&self, started: &Started) -> Option<Option<usize>> {
        let passes = self.filter.came
            && (!self.filter.new_only || started.first_seen())
            && !self.log.hosted(started)
            && admitted(self.only, &self.hidden, |rule| self.log.has(started, rule))
            && [
                &**self.log.names.text(started.name),
                self.log.kept(started.command),
                self.log.kept(started.path),
            ]
            .iter()
            .any(|field| contains(field, &self.text));
        passes.then(|| self.log.came_group(self.groups, started))
    }

    fn judge_went(&self, ended: &Ended) -> Option<Option<usize>> {
        let passes = self.filter.went
            && !self.filter.new_only
            && !ended.came_at().is_some_and(|_| {
                self.log
                    .started(ended.instance())
                    .is_some_and(|started| self.log.hosted(started))
            })
            && admitted(self.only, &self.hidden, |rule| self.log.went_has(ended, rule))
            && contains(went_name(self.log, ended).as_deref().unwrap_or_default(), &self.text);
        passes.then(|| self.log.went_group(self.groups, ended))
    }

    fn admits_came(&self, started: &Started) -> bool {
        self.grade_came(started).is_some_and(|group| self.shown(group))
    }

    fn admits_went(&self, ended: &Ended) -> bool {
        self.grade_went(ended).is_some_and(|group| self.shown(group))
    }

    fn hue(&self, group: Option<usize>) -> Option<Hue> {
        self.groups.hue(group)
    }
}

struct Walk<'a> {
    sieve: Sieve<'a>,
    rows: Vec<(u64, Light<'a>)>,
    series: HashMap<(ProcessInstance, Folder), usize>,
}

impl<'a> Walk<'a> {
    fn came(&mut self, started: &'a Started) {
        let sieve = &self.sieve;
        if !sieve.admits_came(started) {
            return;
        }
        let (at, instance) = (started.at, started.instance());
        let launched = started.launched().zip(started.folder).filter(|_| sieve.filter.series);
        let Some(((launcher, _), folder)) = launched else {
            self.rows.push((at, Light::Came(instance)));
            return;
        };
        let key = (launcher, folder);
        let log = sieve.log;
        match self.series.get(&key) {
            Some(&index) => {
                if let (_, Light::Series(gathered)) = &mut self.rows[index] {
                    gathered.add(log, started);
                }
            }
            None => {
                self.series.insert(key, self.rows.len());
                self.rows.push((at, Light::Series(Gathered::new(log, started))));
            }
        }
    }

    fn went(&mut self, ended: &Ended) {
        if self.sieve.admits_went(ended) {
            self.rows.push((ended.at, Light::Went(ended.instance())));
        }
    }

    fn finish(self) -> Vec<(u64, Light<'a>)> {
        let mut rows = Vec::with_capacity(self.rows.len());
        for (at, light) in self.rows {
            match light {
                Light::Series(gathered) if gathered.count < Pace::SeriesLeast => {
                    rows.extend(
                        gathered
                            .members
                            .into_iter()
                            .map(|member| (member.at, Light::Came(member.instance()))),
                    );
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

pub fn row(log: &Log, groups: &[Group], key: ProcessInstance, clock: fn(u64) -> Clock) -> Option<ActivityRow> {
    let light = if log.started(key).is_some() {
        Light::Came(key)
    } else {
        Light::Went(key)
    };
    let groups = Groups::new(log, groups);
    Rows::new(log, &groups, clock, 0).row(light)
}

pub fn view(log: &Log, ask: &Ask<'_>) -> ActivityView {
    let clock = ask.clock;
    let groups = Groups::new(log, ask.groups);
    let asked = log.ask(ask.filter, ask.groups);
    let frame = Frame::of(log, ask.now, ask.span);
    let (scatter, legend) = scatter(&Sieve::new(log, ask.filter, &groups, None, asked), &frame, clock);
    let scatter = Scatter {
        area: ask.area,
        ..scatter
    };

    let (from, to) = ask
        .area
        .map_or((frame.start, frame.end), |area| (area.from.max(frame.start), area.to.min(frame.end)));

    let window = |at: u64| (from..to).contains(&at);
    let (mut came_count, mut went_count) = (0, 0);
    let mut walk = Walk {
        sieve: Sieve::new(log, ask.filter, &groups, ask.area, asked),
        rows: Vec::new(),
        series: HashMap::new(),
    };
    for event in log.span(from, to).rev() {
        match event {
            Event::Came(started) => {
                came_count += 1;
                walk.came(started);
            }
            Event::Went(ended) => {
                went_count += 1;
                if !(ask.filter.came && ended.came_at().is_some_and(window)) {
                    walk.went(ended);
                }
            }
        }
    }
    let mut rows = walk.finish();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.key().cmp(&a.1.key())));
    let earlier = rows.len().saturating_sub(Pace::Shown);
    rows.truncate(Pace::Shown);
    let mut built = Rows {
        kept: log.shown.take(),
        ..Rows::new(log, &groups, clock, asked)
    };
    let rows = rows.into_iter().filter_map(|(_, row)| built.row(row)).collect();
    log.shown.replace(built.shown);

    ActivityView {
        scatter: Arc::new(scatter),
        rows,
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

    use app_contracts::features::activity::{ActivityRow, Fate, Hue, Launcher, Look};
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
        let mut hues: Vec<(u32, Option<Hue>)> = view.scatter.dots().map(|dot| (dot.key().pid, dot.hue())).collect();
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
    fn a_pid_taken_again_does_not_rename_what_its_first_owner_launched() {
        let mut log = Log::default();
        log.note_running(&[running(4, "Code.exe", at(0, 0))]);
        log.record(&[came(20, 4, "git.exe", at(10, 0))]);
        log.note_running(&[running(4, "notepad.exe", at(30, 0))]);
        log.record(&[came(21, 4, "rg.exe", at(31, 0))]);

        let rows = rows(&log);
        let launchers: Vec<_> = rows
            .iter()
            .map(|row| (only_came(row).name.clone(), only_came(row).launcher.clone(), only_came(row).chain.clone()))
            .collect();
        assert_eq!(
            launchers,
            [
                (
                    Arc::from("rg.exe"),
                    Launcher::Process("notepad.exe".into()),
                    vec![Arc::from("notepad.exe"), Arc::from("rg.exe")]
                ),
                (
                    Arc::from("git.exe"),
                    Launcher::Process("Code.exe".into()),
                    vec![Arc::from("Code.exe"), Arc::from("git.exe")]
                ),
            ]
        );
        let hidden = Filter {
            hidden: vec![Pick::Under("code.exe".into())],
            ..Filter::default()
        };
        let left: Vec<_> = look(&log, &hidden, None).rows.iter().map(|row| only_came(row).name.clone()).collect();
        assert_eq!(left, [Arc::from("rg.exe")]);
    }

    fn listed(log: &Log, filter: &Filter) -> Vec<Arc<str>> {
        look(log, filter, None)
            .rows
            .iter()
            .map(|row| match row {
                ActivityRow::Came(came) => came.name.clone(),
                ActivityRow::Went(went) => went.name.clone().unwrap_or_default(),
                ActivityRow::Series(series) => series.folder.clone(),
            })
            .collect()
    }

    #[test]
    fn a_launcher_named_after_a_look_hides_what_it_launched_from_then_on() {
        let mut log = Log::default();
        log.record(&[came(20, 4, "git.exe", at(10, 0))]);
        let filter = Filter {
            hidden: vec![Pick::Under("code.exe".into())],
            ..Filter::default()
        };
        assert_eq!(listed(&log, &filter), [Arc::from("git.exe")]);

        log.note_running(&[running(4, "Code.exe", at(0, 0))]);

        assert_eq!(listed(&log, &filter), [] as [Arc<str>; 0]);
    }

    #[test]
    fn a_pid_found_taken_before_the_child_started_moves_the_child_after_a_look() {
        let mut log = Log::default();
        log.note_running(&[running(4, "Code.exe", at(0, 0))]);
        log.record(&[came(20, 4, "git.exe", at(10, 0))]);
        let filter = Filter {
            hidden: vec![Pick::Under("notepad.exe".into())],
            ..Filter::default()
        };
        assert_eq!(listed(&log, &filter), [Arc::from("git.exe")]);

        log.note_running(&[running(4, "notepad.exe", at(9, 0))]);

        assert_eq!(listed(&log, &filter), [] as [Arc<str>; 0]);
    }

    #[test]
    fn an_exit_heard_before_its_start_is_judged_by_the_start_once_it_is_heard() {
        let mut log = Log::default();
        log.record(&[went(20, at(10, 2))]);
        let filter = Filter {
            came: false,
            hidden: vec![Pick::Exe("tool.exe".into())],
            ..Filter::default()
        };
        assert_eq!(listed(&log, &filter).len(), 1);

        log.record(&[came(20, 1, "tool.exe", at(10, 0))]);

        assert_eq!(listed(&log, &filter), [] as [Arc<str>; 0]);
    }

    #[test]
    fn a_parent_heard_after_its_child_and_a_look_takes_the_child_under_it() {
        let mut log = Log::default();
        log.record(&[came(21, 20, "git.exe", at(10, 1))]);
        let filter = Filter {
            hidden: vec![Pick::Under("cmd.exe".into())],
            ..Filter::default()
        };
        assert_eq!(listed(&log, &filter), [Arc::from("git.exe")]);

        log.record(&[came(20, 1, "cmd.exe", at(10, 0))]);

        assert_eq!(listed(&log, &filter), [Arc::from("cmd.exe")]);
    }

    fn came_row(rows: &[ActivityRow], pid: u32) -> Arc<app_contracts::features::activity::Came> {
        let found = rows.iter().find_map(|row| match row {
            ActivityRow::Came(came) if came.key == id(pid) => Some(came.clone()),
            _ => None,
        });
        assert!(found.is_some(), "no row for {pid}: {rows:#?}");
        found.unwrap()
    }

    #[test]
    fn a_row_that_did_not_change_is_handed_over_as_it_was() {
        let mut log = Log::default();
        git_burst(&mut log, 3);
        log.record(&[came(20, 1, "a.exe", at(30, 0))]);

        let (first, second) = (rows(&log), rows(&log));

        assert!(Arc::ptr_eq(&came_row(&first, 20), &came_row(&second, 20)));
        let (first, second) = (only_series(&first[1..]), only_series(&second[1..]));
        assert!(
            first.members.iter().zip(&second.members).all(|(a, b)| Arc::ptr_eq(a, b)),
            "{first:#?}"
        );
    }

    #[test]
    fn a_row_is_built_again_once_its_process_went() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0))]);
        assert_eq!(came_row(&rows(&log), 20).exit, None);

        log.record(&[went(20, at(10, 5))]);

        assert!(came_row(&rows(&log), 20).exit.is_some());
    }

    #[test]
    fn a_row_is_built_again_once_the_groups_change() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0))]);
        assert_eq!(came_row(&rows(&log), 20).hue, None);

        let groups = [group("mine", Hue::Purple, vec![Pick::Exe("a.exe".into())])];
        let view = seen(&log, &Filter::default(), &groups, None);

        assert_eq!(came_row(&view.rows, 20).hue, Some(Hue::Purple));
    }

    #[test]
    fn a_row_is_built_again_once_its_launcher_is_named() {
        let mut log = Log::default();
        log.record(&[came(20, 4, "a.exe", at(10, 0))]);
        assert_eq!(came_row(&rows(&log), 20).launcher, Launcher::Unknown);

        log.note_running(&[running(4, "Code.exe", at(0, 0))]);

        let row = came_row(&rows(&log), 20);
        assert_eq!(row.launcher, Launcher::Process("Code.exe".into()));
        assert_eq!(row.chain, [Arc::from("Code.exe"), Arc::from("a.exe")]);
    }

    #[test]
    fn what_a_start_and_its_exit_said_is_read_back_whole() {
        let mut log = Log::default();
        log.record(&[
            ProcessEvent::Came(ProcessCame {
                instance: id(20),
                parent: id(1),
                at: at(10, 0),
                image_path: r"C:\Tools\Tool.exe".into(),
                command_line: r#""C:\Tools\Tool.exe" --go"#.into(),
                working_dir: r"D:\work".into(),
                user: r"HOST\someone".into(),
                session_id: 3,
                elevated: Some(true),
                ..Default::default()
            }),
            ProcessEvent::Came(ProcessCame {
                instance: id(21),
                parent: id(1),
                at: at(11, 0),
                image_path: r"C:\Tools\other.exe".into(),
                elevated: None,
                ..Default::default()
            }),
            ProcessEvent::Went(ProcessWent {
                instance: id(20),
                at: at(10, 2),
                exit_code: 0xC000_0005,
                cpu_cycles: 1 << 40,
                io_read_bytes: 4096,
                io_write_bytes: 0,
                peak_commit_bytes: 300_000_000,
                ..Default::default()
            }),
        ]);

        let rows = rows(&log);
        let tool = came_row(&rows, 20);
        assert_eq!(
            (&*tool.image_path, &*tool.command_line, &*tool.working_dir, &*tool.user),
            (r"C:\Tools\Tool.exe", r#""C:\Tools\Tool.exe" --go"#, r"D:\work", r"HOST\someone")
        );
        assert_eq!((tool.session_id, tool.elevated), (3, Some(true)));
        let exit = tool.exit.as_ref().expect("it went");
        assert_eq!(
            (exit.code, exit.cpu_cycles, exit.read_bytes, exit.write_bytes, exit.peak_commit_bytes),
            (0xC000_0005, 1 << 40, 4096, 0, 300_000_000)
        );
        assert_eq!(came_row(&rows, 21).elevated, None);
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
        assert_eq!(view.scatter.dots().count(), 1, "{:#?}", view.scatter);
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
        assert!(dot(&view.scatter, 100).faint() && dot(&view.scatter, 119).faint(), "{:#?}", view.scatter);
        assert!(!dot(&view.scatter, 20).faint());
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
        let mut pids: Vec<u32> = scatter.dots().map(|dot| dot.key().pid).collect();
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
        let found = scatter.dots().find(|dot| dot.key() == id(pid));
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
        assert_eq!(scatter.dots().count(), 4, "{scatter:#?}");
        assert_eq!((scatter.now, scatter.length), (at(59, 59), HOUR));
        assert_eq!(scatter.now_clock, utc(at(59, 59)));
        let (short, long, alive) = (dot(&scatter, 20), dot(&scatter, 21), dot(&scatter, 22));
        assert_eq!((short.at(), long.at(), alive.at()), (at(15, 0), at(30, 0), at(45, 0)));
        assert_eq!(short.lived(), Lived::For(Ticks::Second));
        assert_eq!(long.lived(), Lived::For(10 * Ticks::Minute));
        assert_eq!(alive.lived(), Lived::Running);
        let orphan = dot(&scatter, 30);
        assert_eq!((orphan.at(), orphan.lived()), (at(50, 0), Lived::Unknown));
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
    fn an_event_that_arrives_late_takes_its_place_by_time() {
        let mut log = Log::default();
        log.record(&[came(21, 1, "b.exe", at(20, 0)), came(23, 1, "d.exe", at(40, 0))]);
        log.record(&[came(22, 1, "c.exe", at(30, 0)), came(20, 1, "a.exe", at(10, 0))]);
        let area = Area {
            from: at(15, 0),
            to: at(35, 0),
            shortest: Lived::Unknown,
            longest: Lived::Running,
        };

        let names: Vec<_> = rows(&log).iter().map(|row| only_came(row).name.clone()).collect();
        assert_eq!(names, ["d.exe", "c.exe", "b.exe", "a.exe"].map(Arc::from));
        let inside: Vec<_> = look(&log, &Filter::default(), Some(area))
            .rows
            .iter()
            .map(|row| only_came(row).name.clone())
            .collect();
        assert_eq!(inside, ["c.exe", "b.exe"].map(Arc::from));
    }

    #[test]
    fn an_exit_heard_before_its_start_still_makes_one_row() {
        let mut log = Log::default();
        log.record(&[went(20, at(10, 2))]);
        log.record(&[came(20, 1, "tool.exe", at(10, 0))]);

        let view = look(&log, &Filter::default(), None);
        assert_eq!(view.rows.len(), 1, "{:#?}", view.rows);
        assert_eq!(only_came(&view.rows[0]).exit.as_ref().map(|exit| exit.lived), Some(2 * Ticks::Second));
        assert_eq!(dot(&view.scatter, 20).lived(), Lived::For(2 * Ticks::Second));
    }

    #[test]
    fn history_heard_after_live_events_goes_before_them() {
        let mut log = Log::default();
        log.note_running(&[running(4, "Code.exe", at(0, 0))]);
        log.record(&[came(30, 4, "git.exe", at(40, 0))]);
        log.history(
            at(1, 0),
            &[
                came(20, 4, "git.exe", at(10, 0)),
                came(21, 4, "git.exe", at(20, 0)),
                went(20, at(10, 1)),
            ],
        );

        let rows = rows(&log);
        let series = only_series(&rows);
        assert_eq!((series.count, series.at, series.key), (3, utc(at(40, 0)), id(20)));
        assert_eq!(series.went, 1);
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

    fn seen_at(log: &Log, now: u64, filter: &Filter, groups: &[Group]) -> ActivityView {
        view(
            log,
            &Ask {
                now,
                span: Span::Hour,
                filter,
                groups,
                area: None,
                clock: utc,
            },
        )
    }

    fn piece_of(view: &ActivityView, pid: u32) -> Arc<Vec<Dot>> {
        let piece = view
            .scatter
            .pieces
            .iter()
            .find(|piece| piece.dots().any(|dot| dot.key() == id(pid)));
        assert!(piece.is_some(), "no piece holds {pid}: {:#?}", view.scatter);
        piece.unwrap().dots.clone()
    }

    #[test]
    fn a_minute_nothing_touched_is_handed_over_as_it_was() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0)), went(20, at(10, 1)), came(21, 1, "b.exe", at(20, 0))]);
        let before = seen_at(&log, at(30, 0), &Filter::default(), &[]);

        log.record(&[came(22, 1, "c.exe", at(30, 1))]);
        let after = seen_at(&log, at(30, 2), &Filter::default(), &[]);

        assert!(Arc::ptr_eq(&piece_of(&before, 20), &piece_of(&after, 20)));
        assert!(Arc::ptr_eq(&piece_of(&before, 21), &piece_of(&after, 21)));
        assert_eq!(drawn(&after.scatter), [20, 21, 22]);
    }

    #[test]
    fn a_minute_whose_process_exits_later_is_drawn_again() {
        let mut log = Log::default();
        log.record(&[came(20, 1, "a.exe", at(10, 0)), came(21, 1, "b.exe", at(20, 0))]);
        let before = seen_at(&log, at(30, 0), &Filter::default(), &[]);

        log.record(&[went(21, at(30, 1))]);
        let after = seen_at(&log, at(30, 2), &Filter::default(), &[]);

        assert!(Arc::ptr_eq(&piece_of(&before, 20), &piece_of(&after, 20)));
        assert_eq!(dot(&after.scatter, 21).lived(), Lived::For(10 * Ticks::Minute + Ticks::Second));
    }

    #[test]
    fn a_minute_hands_each_look_over_as_one_run_in_time_order() {
        let mut log = Log::default();
        log.record(&[
            came(20, 1, "a.exe", at(10, 1)),
            came(21, 1, "b.exe", at(10, 2)),
            went(21, at(10, 3)),
            went(30, at(10, 4)),
            came(22, 1, "c.exe", at(10, 5)),
        ]);

        let view = look(&log, &Filter::default(), None);

        let runs: Vec<(Look, Vec<u32>)> = view
            .scatter
            .pieces
            .iter()
            .flat_map(|piece| {
                piece
                    .runs
                    .iter()
                    .map(move |run| (run.look, piece.run(run).iter().map(|dot| dot.key().pid).collect()))
            })
            .collect();
        let plain = |fate| Look {
            fate,
            hue: None,
            faint: false,
        };
        assert_eq!(
            runs,
            [
                (plain(Fate::Unknown), vec![30]),
                (plain(Fate::Ended), vec![21]),
                (plain(Fate::Running), vec![20, 22]),
            ]
        );
    }

    struct Dice(u64);

    impl Dice {
        fn roll(&mut self, sides: u64) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 % sides
        }
    }

    enum Step {
        Record(Vec<ProcessEvent>),
        History(u64, Vec<ProcessEvent>),
        Running(Vec<WindowsProcessStats>),
    }

    fn replayed(steps: &[Step]) -> Log {
        let mut log = Log::default();
        for step in steps {
            match step {
                Step::Record(events) => log.record(events),
                Step::History(since, events) => log.history(*since, events),
                Step::Running(processes) => log.note_running(processes),
            }
        }
        log
    }

    #[test]
    fn a_view_built_on_earlier_views_draws_what_a_fresh_log_draws() {
        let images = ["taskhostw.exe", "git.exe", "conhost.exe", "cmd.exe", "rg.exe"];
        let filters = [
            Filter::default(),
            singles(),
            Filter {
                hidden: vec![Pick::Exe("git.exe".into())],
                ..Filter::default()
            },
            Filter {
                came: false,
                ..Filter::default()
            },
        ];
        let groups = [group("tools", Hue::Purple, vec![Pick::Under("claude.exe".into())])];
        let mut dice = Dice(0x9e37_79b9_7f4a_7c15);
        let mut log = Log::default();
        let mut steps = Vec::new();
        let mut alive: Vec<(u32, u64, &str)> = Vec::new();
        let mut now = at(0, 0);
        let mut filter = 0;

        let claude = vec![running(4, "claude.exe", at(0, 0) - HOUR)];
        log.note_running(&claude);
        steps.push(Step::Running(claude));
        let earlier = vec![came(10, 4, "git.exe", at(0, 0) - Ticks::Minute), went(10, at(0, 0) - Ticks::Second)];
        log.history(at(0, 0) - HOUR, &earlier);
        steps.push(Step::History(at(0, 0) - HOUR, earlier));

        for round in 0..240u32 {
            now += (1 + dice.roll(40)) * Ticks::Second;
            let mut events = Vec::new();
            for start in 0..dice.roll(4) {
                let pid = 100 + round * 8 + start as u32;
                let parent = match dice.roll(3) {
                    0 if !alive.is_empty() => alive[dice.roll(alive.len() as u64) as usize].0,
                    1 => 1,
                    _ => 4,
                };
                let image = images[dice.roll(images.len() as u64) as usize];
                let when = now - dice.roll(3) * Ticks::Second;
                events.push(came(pid, parent, image, when));
                alive.push((pid, when, image));
            }
            for _ in 0..dice.roll(3) {
                if alive.is_empty() {
                    break;
                }
                let (pid, started, _) = alive.swap_remove(dice.roll(alive.len() as u64) as usize);
                events.push(went(pid, (now - dice.roll(2) * Ticks::Second).max(started + 1)));
            }
            if dice.roll(6) == 0 {
                events.push(went(5000 + round, now));
            }
            log.record(&events);
            steps.push(Step::Record(events));
            if dice.roll(5) == 0 {
                let mut processes = vec![running(4, "claude.exe", at(0, 0) - HOUR)];
                processes.extend(alive.iter().map(|&(pid, started, image)| running(pid, image, started)));
                log.note_running(&processes);
                steps.push(Step::Running(processes));
            }
            if dice.roll(12) == 0 {
                filter = dice.roll(filters.len() as u64) as usize;
            }

            let built = seen_at(&log, now, &filters[filter], &groups);
            let expected = seen_at(&replayed(&steps), now, &filters[filter], &groups);
            let dots = |view: &ActivityView| view.scatter.dots().copied().collect::<Vec<_>>();
            assert_eq!(dots(&built), dots(&expected), "round {round}");
            assert_eq!(built.legend, expected.legend, "round {round}");
        }
    }
}
