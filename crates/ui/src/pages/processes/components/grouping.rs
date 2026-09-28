use amethystate::ReactiveMap;
use app_contracts::features::agents::EnvironmentKind;
use app_contracts::features::processes::{
    HostedService, PinnedProcess, ProcessCategory, ProcessColumn, ProcessRow, ProcessWindow,
    WslEnvironment, WslProcess,
};

use super::Step;
use crate::theme::size;
use crate::widgets::table_cell::Highlight;

pub(crate) struct ProcessName;

#[expect(non_upper_case_globals)]
impl ProcessName {
    pub(crate) const MemoryCompression: &str = "Memory Compression";
    const ServiceHost: &str = "svchost.exe";
    const WslHost: &str = "vmmemWSL";
}
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) struct ProcessGroup {
    leader: ProcessRow,
    members: Vec<ProcessRow>,
}

pub(crate) fn is_service_host(row: &ProcessRow) -> bool {
    row.name.eq_ignore_ascii_case(ProcessName::ServiceHost)
}

type GroupKey = (Arc<str>, Option<u32>);

fn group_key(row: &ProcessRow) -> GroupKey {
    let alone = is_service_host(row).then_some(row.pid);
    (row.name.clone(), alone)
}

pub(crate) fn group_by_name(rows: &[ProcessRow], leader_pid: Option<u32>) -> Vec<ProcessGroup> {
    let mut order: Vec<GroupKey> = Vec::new();
    let mut groups: HashMap<GroupKey, Vec<ProcessRow>> = HashMap::new();
    for row in rows {
        let key = group_key(row);
        groups
            .entry(key.clone())
            .or_insert_with(|| {
                order.push(key);
                Vec::new()
            })
            .push(row.clone());
    }

    order
        .into_iter()
        .filter_map(|name| {
            let mut members = groups.remove(&name)?;
            if members.is_empty() {
                return None;
            }
            members.sort_unstable_by_key(|r| r.pid);
            let leader_idx = leader_pid
                .and_then(|pid| members.iter().position(|r| r.pid == pid))
                .unwrap_or(0);

            let leader = if members.len() > 1 {
                ProcessRow {
                    cpu_percent: members.iter().map(|r| r.cpu_percent).sum(),
                    memory_bytes: members.iter().map(|r| r.memory_bytes).sum(),
                    disk_bytes: members.iter().map(|r| r.disk_bytes).sum(),
                    net_bytes: members.iter().map(|r| r.net_bytes).sum(),
                    ..members[leader_idx].clone()
                }
            } else {
                members[0].clone()
            };

            Some(ProcessGroup { leader, members })
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SectionId {
    Pinned,
    Category(ProcessCategory),
}

impl SectionId {
    const PINNED: &str = "pinned";

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Pinned => Self::PINNED,
            Self::Category(category) => category.id(),
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        if id == Self::PINNED {
            return Some(Self::Pinned);
        }
        ProcessCategory::from_id(id).map(Self::Category)
    }
}

impl guinea::Mark for SectionId {
    fn name(&self) -> &'static str {
        self.id()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct SectionOrder(Vec<SectionId>);

impl Default for SectionOrder {
    fn default() -> Self {
        Self::ranked(|_| None)
    }
}

impl SectionOrder {
    pub(crate) fn ranked(rank: impl Fn(SectionId) -> Option<u32>) -> Self {
        let mut ids: Vec<SectionId> = std::iter::once(SectionId::Pinned)
            .chain(ProcessCategory::ORDER.map(SectionId::Category))
            .collect();
        ids.sort_by_key(|id| rank(*id).unwrap_or(u32::MAX));
        Self(ids)
    }

    pub(crate) fn kept(ranks: Option<&ReactiveMap<String, u32>>) -> Self {
        ranks
            .map(|ranks| Self::ranked(|id| ranks.get(id.id())))
            .unwrap_or_default()
    }

    pub(crate) fn ids(&self) -> &[SectionId] {
        &self.0
    }

    pub(crate) fn can_shift(&self, section: SectionId, step: Step) -> bool {
        self.shifted(section, step).is_some()
    }

    pub(crate) fn shifted(&self, section: SectionId, step: Step) -> Option<Self> {
        let at = self.0.iter().position(|id| *id == section)?;
        let before = match step {
            Step::Up => Some(self.0[at.checked_sub(1)?]),
            Step::Down if at + 1 < self.0.len() => self.0.get(at + 2).copied(),
            Step::Down => return None,
        };
        Some(self.moved(section, before))
    }

    pub(crate) fn store(&self, ranks: &ReactiveMap<String, u32>) {
        for (rank, id) in self.0.iter().enumerate() {
            if let Err(err) = ranks.insert(id.id().to_string(), &(rank as u32)) {
                tracing::warn!(section = id.id(), ?err, "section order write failed");
            }
        }
    }

    pub(crate) fn forget(ranks: &ReactiveMap<String, u32>) {
        if let Err(err) = ranks.clear() {
            tracing::warn!(?err, "section order reset failed");
        }
    }

    pub(crate) fn moved(&self, section: SectionId, before: Option<SectionId>) -> Self {
        let mut ids: Vec<SectionId> = self.0.iter().copied().filter(|id| *id != section).collect();
        let at = before
            .and_then(|before| ids.iter().position(|id| *id == before))
            .unwrap_or(ids.len());
        ids.insert(at, section);
        Self(ids)
    }
}

pub(crate) struct Environment {
    pid_ns: u64,
    kind: EnvironmentKind,
    leader: ProcessRow,
    processes: Vec<WslProcess>,
}

impl Environment {
    fn of(environment: &WslEnvironment) -> Self {
        let processes = environment.processes.to_vec();
        let leader = ProcessRow {
            pid: 0,
            name: environment.name.clone(),
            display_name: environment.name.clone(),
            cpu_percent: processes.iter().map(|p| p.row.cpu_percent).sum(),
            memory_bytes: processes.iter().map(|p| p.row.memory_bytes).sum(),
            disk_bytes: processes.iter().map(|p| p.row.disk_bytes).sum(),
            net_bytes: processes.iter().map(|p| p.row.net_bytes).sum(),
            exe_path: "".into(),
            package_full_name: "".into(),
            owner: None,
            owner_pid: None,
            category: ProcessCategory::Wsl,
            services: None,
            windows: None,
        };
        Self {
            pid_ns: environment.pid_ns,
            kind: environment.kind,
            leader,
            processes,
        }
    }
}

pub(crate) fn environment_key(pid_ns: u64) -> String {
    format!("wsl/{pid_ns}")
}

fn environments_of(wsl: &[WslEnvironment]) -> Vec<Environment> {
    wsl.iter().map(Environment::of).collect()
}

fn is_wsl_host(group: &ProcessGroup) -> bool {
    group.leader.name.eq_ignore_ascii_case(ProcessName::WslHost)
}

pub(crate) struct Section {
    pub(crate) id: SectionId,
    headed: bool,
    ruled: bool,
    groups: Vec<ProcessGroup>,
    absent: Vec<(Arc<str>, PinnedProcess)>,
    environments: Vec<Environment>,
    host: Option<ProcessRow>,
    environments_under: Option<u32>,
    consoles: HashMap<u32, Vec<ProcessRow>>,
}

pub(crate) type Pins = HashMap<Arc<str>, PinnedProcess>;

impl Section {
    fn empty(id: SectionId, headed: bool) -> Self {
        Self {
            id,
            headed,
            ruled: false,
            groups: Vec::new(),
            absent: Vec::new(),
            environments: Vec::new(),
            host: None,
            environments_under: None,
            consoles: HashMap::new(),
        }
    }

    fn hosts_environments(&self, pid: u32) -> bool {
        self.environments_under == Some(pid) && !self.environments.is_empty()
    }

    fn new(id: SectionId, headed: bool, groups: Vec<ProcessGroup>) -> Option<Self> {
        Self::with_absent(id, headed, groups, Vec::new())
    }

    fn with_absent(
        id: SectionId,
        headed: bool,
        groups: Vec<ProcessGroup>,
        absent: Vec<(Arc<str>, PinnedProcess)>,
    ) -> Option<Self> {
        (!groups.is_empty() || !absent.is_empty()).then(|| Self {
            groups,
            absent,
            ..Self::empty(id, headed)
        })
    }

    fn wsl(host: Option<ProcessRow>, environments: Vec<Environment>) -> Option<Self> {
        (host.is_some() || !environments.is_empty()).then(|| Self {
            host,
            environments,
            ..Self::empty(SectionId::Category(ProcessCategory::Wsl), true)
        })
    }
}

fn is_pinned(group: &ProcessGroup, pins: &Pins) -> bool {
    pins.contains_key(&group.leader.name)
}

fn absent_pins(groups: &[ProcessGroup], pins: &Pins) -> Vec<(Arc<str>, PinnedProcess)> {
    let mut absent: Vec<(Arc<str>, PinnedProcess)> = pins
        .iter()
        .filter(|(name, _)| !groups.iter().any(|group| group.leader.name == **name))
        .map(|(name, pin)| (name.clone(), pin.clone()))
        .collect();
    absent.sort_by_key(|(name, _)| name.to_lowercase());
    absent
}

fn one_section(groups: Vec<ProcessGroup>, pins: &Pins, wsl: &[WslEnvironment]) -> Vec<Section> {
    let absent = absent_pins(&groups, pins);
    let (pinned, rest): (Vec<_>, Vec<_>) = groups.into_iter().partition(|group| is_pinned(group, pins));
    let rest_id = SectionId::Category(ProcessCategory::ORDER[0]);
    let mut sections: Vec<Section> = [
        Section::with_absent(SectionId::Pinned, false, pinned, absent),
        Section::new(rest_id, false, rest),
    ]
    .into_iter()
    .flatten()
    .collect();
    let environments = environments_of(wsl);
    if !environments.is_empty() {
        let host = sections.iter().enumerate().find_map(|(at, section)| {
            section
                .groups
                .iter()
                .find(|group| is_wsl_host(group) && group.members.len() == 1)
                .map(|group| (at, group.leader.pid))
        });
        let section = match host {
            Some((at, pid)) => {
                sections[at].environments_under = Some(pid);
                &mut sections[at]
            }
            None => {
                if !sections.last().is_some_and(|section| section.id == rest_id) {
                    sections.push(Section::empty(rest_id, false));
                }
                sections.last_mut().expect("the rest was just made sure of")
            }
        };
        section.environments = environments;
    }
    if let [pinned, _] = sections.as_mut_slice() {
        pinned.ruled = true;
    }
    sections
}

#[cfg(test)]
pub(crate) fn split_by_category(groups: Vec<ProcessGroup>) -> Vec<Section> {
    split_keeping(groups, &Pins::new(), &[], None, &SectionOrder::default())
}

fn split_keeping(
    groups: Vec<ProcessGroup>,
    pins: &Pins,
    wsl: &[WslEnvironment],
    keep: Option<(u32, SectionId)>,
    order: &SectionOrder,
) -> Vec<Section> {
    let mut absent = absent_pins(&groups, pins);
    let (hosts, groups): (Vec<_>, Vec<_>) = groups.into_iter().partition(is_wsl_host);
    let mut wsl_section = Section::wsl(hosts.into_iter().next().map(|host| host.leader), environments_of(wsl));
    let mut by_section: HashMap<SectionId, Vec<ProcessGroup>> = HashMap::new();
    for group in groups {
        let id = if is_pinned(&group, pins) {
            SectionId::Pinned
        } else {
            match keep {
                Some((pid, SectionId::Category(kept)))
                    if group.members.iter().any(|member| member.pid == pid) =>
                {
                    SectionId::Category(kept)
                }
                _ => SectionId::Category(group_category(&group)),
            }
        };
        by_section.entry(id).or_default().push(group);
    }

    order
        .ids()
        .iter()
        .filter_map(|id| match id {
            SectionId::Pinned => Section::with_absent(
                *id,
                true,
                by_section.remove(id).unwrap_or_default(),
                std::mem::take(&mut absent),
            ),
            SectionId::Category(ProcessCategory::Wsl) => wsl_section.take(),
            SectionId::Category(_) => Section::new(*id, true, by_section.remove(id)?),
        })
        .collect()
}

pub(crate) fn detach_consoles(rows: &[ProcessRow]) -> (Vec<ProcessRow>, HashMap<u32, Vec<ProcessRow>>) {
    let present: HashSet<u32> = rows.iter().map(|r| r.pid).collect();
    let mut rest = Vec::with_capacity(rows.len());
    let mut consoles: HashMap<u32, Vec<ProcessRow>> = HashMap::new();
    for row in rows {
        match row.owner_pid.filter(|owner| *owner != row.pid && present.contains(owner)) {
            Some(owner) => consoles.entry(owner).or_default().push(row.clone()),
            None => rest.push(row.clone()),
        }
    }
    for list in consoles.values_mut() {
        list.sort_unstable_by_key(|r| r.pid);
    }
    (rest, consoles)
}

pub(crate) fn attach_consoles(sections: &mut [Section], mut consoles: HashMap<u32, Vec<ProcessRow>>) {
    for section in sections {
        for group in &section.groups {
            for member in &group.members {
                if let Some(list) = consoles.remove(&member.pid) {
                    section.consoles.insert(member.pid, list);
                }
            }
        }
    }
}

fn group_category(group: &ProcessGroup) -> ProcessCategory {
    let rank = |c: ProcessCategory| {
        ProcessCategory::ORDER
            .iter()
            .position(|o| *o == c)
            .unwrap_or(usize::MAX)
    };
    group
        .members
        .iter()
        .map(|m| m.category)
        .min_by_key(|c| rank(*c))
        .unwrap_or(group.leader.category)
}

#[derive(Default, Clone, Copy)]
struct SectionTotals {
    cpu_percent: f32,
    memory_bytes: u64,
    disk_bytes: u64,
    net_bytes: u64,
    group_count: usize,
    compressed_bytes: u64,
    idle_cpu_percent: f32,
}

impl SectionTotals {
    fn add(&mut self, row: &ProcessRow) {
        self.cpu_percent += row.cpu_percent;
        self.memory_bytes += row.memory_bytes;
        self.disk_bytes += row.disk_bytes;
        self.net_bytes += row.net_bytes;
    }

    fn of(section: &Section) -> Self {
        let mut totals = Self::default();
        for group in &section.groups {
            totals.add(&group.leader);
            totals.group_count += 1;

            if &*group.leader.name == ProcessName::MemoryCompression {
                totals.compressed_bytes += group.leader.memory_bytes;
            }
            if is_idle(&group.leader) {
                totals.idle_cpu_percent += group.leader.cpu_percent;
            }
        }
        for console in section.consoles.values().flatten() {
            totals.add(console);
        }
        for environment in &section.environments {
            totals.add(&environment.leader);
        }
        totals.group_count += section.absent.len() + section.environments.len();
        if let Some(host) = &section.host {
            let group_count = totals.group_count;
            totals = Self {
                group_count,
                ..Self::default()
            };
            totals.add(host);
        }
        totals
    }
}

pub(crate) struct ViewState<'a> {
    pub(crate) groups: &'a HashSet<String>,
    pub(crate) processes: &'a HashSet<u32>,
    pub(crate) collapsed_sections: &'a HashSet<SectionId>,
    pub(crate) exited: &'a HashSet<u32>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Child {
    Window(ProcessWindow),
    Service(HostedService),
    Console,
}

impl Child {
    pub(crate) fn has_metrics(&self) -> bool {
        matches!(self, Self::Console)
    }
}

fn children_of(row: &ProcessRow) -> impl Iterator<Item = Child> + '_ {
    let windows = row.windows.iter().flat_map(|titles| titles.iter().cloned().map(Child::Window));
    let services = row.services.iter().flat_map(|services| services.iter().cloned().map(Child::Service));
    windows.chain(services)
}

fn plain_row(row: &ProcessRow, depth: u8) -> DisplayRow {
    DisplayRow {
        row: row.clone(),
        depth,
        has_children: false,
        is_expanded: false,
        group_size: 1,
        section: None,
        highlight: None,
        child: None,
        details: false,
        details_expanded: false,
        exited: false,
        rule_below: false,
        absent: false,
        wsl: None,
        lifted: false,
        drop_edge: None,
    }
}

fn process_row(row: &ProcessRow, depth: u8, expanded: &ViewState<'_>, section: &Section) -> DisplayRow {
    let details = children_of(row).next().is_some()
        || section.consoles.contains_key(&row.pid)
        || section.hosts_environments(row.pid);
    DisplayRow {
        exited: expanded.exited.contains(&row.pid),
        details,
        details_expanded: details && expanded.processes.contains(&row.pid),
        ..plain_row(row, depth)
    }
}

fn child_row(row: &ProcessRow, depth: u8, child: Child) -> DisplayRow {
    DisplayRow {
        child: Some(child),
        ..plain_row(row, depth)
    }
}

fn push_environment(out: &mut Vec<DisplayRow>, environment: &Environment, expanded: &ViewState<'_>, depth: u8) {
    let is_expanded = expanded.groups.contains(&environment_key(environment.pid_ns));
    out.push(DisplayRow {
        has_children: true,
        is_expanded,
        group_size: environment.processes.len(),
        wsl: Some(WslRow::Environment {
            pid_ns: environment.pid_ns,
            kind: environment.kind,
        }),
        ..plain_row(&environment.leader, depth)
    });
    if !is_expanded {
        return;
    }
    for process in &environment.processes {
        out.push(DisplayRow {
            wsl: Some(WslRow::Process {
                global_pid: process.global_pid,
            }),
            ..plain_row(&process.row, depth + 1)
        });
    }
}

fn push_with_details(out: &mut Vec<DisplayRow>, host: DisplayRow, section: &Section, expanded: &ViewState<'_>) {
    let open = host.details_expanded;
    let depth = host.depth + 1;
    let row = host.row.clone();
    out.push(host);
    if !open {
        return;
    }
    for child in children_of(&row) {
        out.push(child_row(&row, depth, child));
    }
    for console in section.consoles.get(&row.pid).into_iter().flatten() {
        out.push(child_row(console, depth, Child::Console));
    }
    if section.hosts_environments(row.pid) {
        for environment in &section.environments {
            push_environment(out, environment, expanded, depth - 1);
        }
    }
}

pub(crate) fn flatten_for_display(sections: &[Section], expanded: &ViewState<'_>) -> Vec<DisplayRow> {
    let mut out = Vec::new();

    for section in sections {
        if section.headed {
            let section_expanded = !expanded.collapsed_sections.contains(&section.id);
            out.push(DisplayRow::section(
                section.id,
                SectionTotals::of(section),
                section_expanded,
            ));
            if !section_expanded {
                continue;
            }
        }

        for group in &section.groups {
            if group.members.len() == 1 {
                push_with_details(&mut out, process_row(&group.leader, 1, expanded, section), section, expanded);
                continue;
            }

            let is_expanded = expanded.groups.contains(&*group.leader.name);
            out.push(DisplayRow {
                has_children: true,
                is_expanded,
                group_size: group.members.len(),
                details: false,
                details_expanded: false,
                exited: group.members.iter().all(|m| expanded.exited.contains(&m.pid)),
                ..process_row(&group.leader, 1, expanded, section)
            });

            if is_expanded {
                for member in &group.members {
                    push_with_details(&mut out, process_row(member, 2, expanded, section), section, expanded);
                }
            }
        }

        if section.environments_under.is_none() {
            for environment in &section.environments {
                push_environment(&mut out, environment, expanded, 1);
            }
        }

        out.extend(section.absent.iter().map(|(name, pin)| DisplayRow::absent(name, pin)));

        if section.ruled
            && let Some(last) = out.last_mut()
        {
            last.rule_below = true;
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Order {
    pub(crate) column: ProcessColumn,
    pub(crate) descending: bool,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            column: ProcessColumn::Cpu,
            descending: true,
        }
    }
}

fn tenths(percent: f32) -> i64 {
    (percent * 10.0).round() as i64
}

fn sorted_memory(row: &ProcessRow) -> Option<u64> {
    (&*row.name != ProcessName::MemoryCompression).then_some(row.memory_bytes)
}

pub(crate) fn is_idle(row: &ProcessRow) -> bool {
    row.pid == 0 && row.category == ProcessCategory::WindowsKernel
}

fn sorted_cpu(row: &ProcessRow) -> Option<i64> {
    (!is_idle(row)).then_some(tenths(row.cpu_percent))
}

fn compare_by(column: ProcessColumn, a: &ProcessRow, b: &ProcessRow) -> Ordering {
    match column {
        ProcessColumn::Name => a
            .display_name
            .chars()
            .flat_map(char::to_lowercase)
            .cmp(b.display_name.chars().flat_map(char::to_lowercase)),
        ProcessColumn::Pid => a.pid.cmp(&b.pid),
        ProcessColumn::ProcessName => a
            .name
            .chars()
            .flat_map(char::to_lowercase)
            .cmp(b.name.chars().flat_map(char::to_lowercase)),
        ProcessColumn::Cpu => sorted_cpu(a).cmp(&sorted_cpu(b)),
        ProcessColumn::Memory => sorted_memory(a).cmp(&sorted_memory(b)),
        ProcessColumn::Disk => a.disk_bytes.cmp(&b.disk_bytes),
        ProcessColumn::Net => a.net_bytes.cmp(&b.net_bytes),
    }
}

fn compare_rows(order: &Order, a: &ProcessRow, b: &ProcessRow) -> Ordering {
    let primary = compare_by(order.column, a, b);
    let primary = if order.descending { primary.reverse() } else { primary };
    primary
        .then_with(|| compare_by(ProcessColumn::Name, a, b))
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.pid.cmp(&b.pid))
}

pub(crate) fn sort_groups(sections: &mut [Section], order: &Order) {
    for section in sections {
        section
            .groups
            .sort_by(|a, b| compare_rows(order, &a.leader, &b.leader));
        for group in &mut section.groups {
            group.members.sort_by(|a, b| compare_rows(order, a, b));
        }
        section
            .environments
            .sort_by(|a, b| compare_rows(order, &a.leader, &b.leader));
        for environment in &mut section.environments {
            environment
                .processes
                .sort_by(|a, b| compare_rows(order, &a.row, &b.row).then(a.global_pid.cmp(&b.global_pid)));
        }
    }
}

pub(crate) struct GroupsCache {
    rows_ptr: *const ProcessRow,
    rows_len: usize,
    wsl_ptr: *const WslEnvironment,
    wsl_len: usize,
    exited: Vec<u32>,
    selected: Option<u32>,
    kept: Option<SectionId>,
    order: Order,
    by_type: bool,
    pins: Pins,
    section_order: SectionOrder,
    sections: Vec<Section>,
}

pub(crate) struct Grouping<'a> {
    pub(crate) selected: Option<u32>,
    pub(crate) kept: Option<SectionId>,
    pub(crate) order: &'a Order,
    pub(crate) by_type: bool,
    pub(crate) pins: &'a Pins,
    pub(crate) wsl: &'a [WslEnvironment],
    pub(crate) section_order: &'a SectionOrder,
}

impl GroupsCache {
    pub(crate) fn empty() -> Self {
        Self {
            rows_ptr: std::ptr::null(),
            rows_len: 0,
            wsl_ptr: std::ptr::null(),
            wsl_len: 0,
            exited: Vec::new(),
            selected: None,
            kept: None,
            order: Order::default(),
            by_type: true,
            pins: Pins::new(),
            section_order: SectionOrder::default(),
            sections: Vec::new(),
        }
    }

    pub(crate) fn get(
        &mut self,
        rows: &[ProcessRow],
        exited: &[ProcessRow],
        grouping: Grouping<'_>,
    ) -> &mut [Section] {
        let Grouping {
            selected,
            kept,
            order,
            by_type,
            pins,
            wsl,
            section_order,
        } = grouping;
        let rows_ptr = rows.as_ptr();
        let same_exited = self.exited.len() == exited.len()
            && self.exited.iter().zip(exited).all(|(pid, row)| *pid == row.pid);
        if self.rows_ptr != rows_ptr
            || self.rows_len != rows.len()
            || self.wsl_ptr != wsl.as_ptr()
            || self.wsl_len != wsl.len()
            || !same_exited
            || self.selected != selected
            || self.kept != kept
            || self.order != *order
            || self.by_type != by_type
            || self.pins != *pins
            || self.section_order != *section_order
        {
            let (mut rest, consoles) = detach_consoles(rows);
            rest.extend(exited.iter().cloned());
            let groups = group_by_name(&rest, selected);
            self.sections = if by_type {
                split_keeping(groups, pins, wsl, selected.zip(kept), section_order)
            } else {
                one_section(groups, pins, wsl)
            };
            self.pins.clone_from(pins);
            self.section_order.clone_from(section_order);
            attach_consoles(&mut self.sections, consoles);
            sort_groups(&mut self.sections, order);
            self.rows_ptr = rows_ptr;
            self.rows_len = rows.len();
            self.wsl_ptr = wsl.as_ptr();
            self.wsl_len = wsl.len();
            self.exited = exited.iter().map(|row| row.pid).collect();
            self.selected = selected;
            self.kept = kept;
            self.order = *order;
            self.by_type = by_type;
        }
        &mut self.sections
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct SectionRow {
    pub(crate) id: SectionId,
    pub(crate) compressed_bytes: u64,
}

#[derive(Clone)]
pub(crate) struct DisplayRow {
    pub(crate) row: ProcessRow,
    pub(crate) depth: u8,
    pub(crate) has_children: bool,
    pub(crate) is_expanded: bool,
    pub(crate) group_size: usize,
    pub(crate) section: Option<SectionRow>,
    pub(crate) highlight: Option<Highlight>,
    pub(crate) child: Option<Child>,
    pub(crate) details: bool,
    pub(crate) details_expanded: bool,
    pub(crate) exited: bool,
    pub(crate) rule_below: bool,
    pub(crate) absent: bool,
    pub(crate) wsl: Option<WslRow>,
    pub(crate) lifted: bool,
    pub(crate) drop_edge: Option<DropEdge>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DropEdge {
    Above,
    Below,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum WslRow {
    Environment { pid_ns: u64, kind: EnvironmentKind },
    Process { global_pid: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Selection {
    Process(u32),
    Group(u32),
    Linux(u32),
}

fn selected_rows(sections: &[Section], selection: Option<Selection>) -> Vec<ProcessRow> {
    let Some(selection) = selection else {
        return Vec::new();
    };
    for section in sections {
        for group in &section.groups {
            match selection {
                Selection::Group(pid) if group.members.iter().any(|m| m.pid == pid) => {
                    return group.members.clone();
                }
                Selection::Process(pid) | Selection::Group(pid) => {
                    if let Some(member) = group.members.iter().find(|m| m.pid == pid) {
                        return vec![member.clone()];
                    }
                }
                Selection::Linux(_) => return Vec::new(),
            }
        }
        if let Selection::Process(pid) = selection
            && let Some(console) = section.consoles.values().flatten().find(|c| c.pid == pid)
        {
            return vec![console.clone()];
        }
    }
    Vec::new()
}

#[derive(Default)]
pub(crate) struct Held {
    selection: Option<Selection>,
    rows: Vec<ProcessRow>,
    section: Option<SectionId>,
}

fn section_of(sections: &[Section], selection: Option<Selection>) -> Option<SectionId> {
    let pid = match selection? {
        Selection::Process(pid) | Selection::Group(pid) => pid,
        Selection::Linux(_) => return None,
    };
    sections
        .iter()
        .find(|section| {
            section
                .groups
                .iter()
                .any(|group| group.members.iter().any(|member| member.pid == pid))
        })
        .map(|section| section.id)
}

impl Held {
    pub(crate) fn section(&self, selection: Option<Selection>) -> Option<SectionId> {
        self.section.filter(|_| selection.is_some() && self.selection == selection)
    }

    pub(crate) fn exited(&self, selection: Option<Selection>, rows: &[ProcessRow]) -> Vec<ProcessRow> {
        if selection.is_none() || self.selection != selection {
            return Vec::new();
        }
        self.rows
            .iter()
            .filter(|gone| !rows.iter().any(|row| row.pid == gone.pid))
            .map(exited_copy)
            .collect()
    }

    pub(crate) fn hold(&mut self, sections: &[Section], selection: Option<Selection>) {
        if self.selection != selection {
            self.section = section_of(sections, selection);
        }
        self.selection = selection;
        self.rows = selected_rows(sections, selection);
    }

    pub(crate) fn row(&self, pid: u32) -> Option<&ProcessRow> {
        self.rows.iter().find(|row| row.pid == pid)
    }
}

fn exited_copy(row: &ProcessRow) -> ProcessRow {
    ProcessRow {
        cpu_percent: 0.0,
        memory_bytes: 0,
        disk_bytes: 0,
        net_bytes: 0,
        services: None,
        windows: None,
        ..row.clone()
    }
}

fn paint_block(block: &mut [DisplayRow]) {
    let last = block.len() - 1;
    for (i, d) in block.iter_mut().enumerate() {
        d.highlight = Some(Highlight {
            top: i == 0,
            bottom: i == last,
        });
    }
}

fn highlight_process(rows: &mut [DisplayRow], pid: u32) {
    let Some(host) = rows
        .iter()
        .position(|d| d.stands_for_one_process() && d.wsl.is_none() && d.row.pid == pid)
    else {
        return;
    };
    let details = if rows[host].details_expanded {
        let depth = rows[host].depth;
        rows[host + 1..]
            .iter()
            .take_while(|d| d.child.is_some() && d.depth > depth)
            .count()
    } else {
        0
    };
    paint_block(&mut rows[host..=host + details]);
}

pub(crate) fn highlight(rows: &mut [DisplayRow], selection: Option<Selection>) {
    let Some(selection) = selection else {
        return;
    };
    if let Selection::Process(pid) = selection {
        highlight_process(rows, pid);
        return;
    }
    if let Selection::Linux(global_pid) = selection {
        let wanted = Some(WslRow::Process { global_pid });
        if let Some(at) = rows.iter().position(|d| d.wsl == wanted) {
            paint_block(&mut rows[at..=at]);
        }
        return;
    }
    let mut at = 0;
    while at < rows.len() {
        let heading = &rows[at];
        let members = if heading.section.is_none() && heading.depth == 1 && heading.is_expanded {
            rows[at + 1..]
                .iter()
                .take_while(|d| d.depth >= 2)
                .count()
        } else {
            0
        };
        let span = &mut rows[at..=at + members];
        if let Selection::Group(pid) = selection
            && span[0].section.is_none()
            && !span[0].absent
            && span[0].wsl.is_none()
            && span[0].row.pid == pid
        {
            paint_block(span);
        }
        at += members + 1;
    }
}

impl DisplayRow {
    fn section(id: SectionId, totals: SectionTotals, is_expanded: bool) -> Self {
        let label = id.id();
        Self {
            row: ProcessRow {
                pid: 0,
                name: label.into(),
                display_name: label.into(),
                cpu_percent: (totals.cpu_percent - totals.idle_cpu_percent).max(0.0),
                memory_bytes: totals.memory_bytes.saturating_sub(totals.compressed_bytes),
                disk_bytes: totals.disk_bytes,
                net_bytes: totals.net_bytes,
                exe_path: "".into(),
                package_full_name: "".into(),
                owner: None,
                owner_pid: None,
                category: ProcessCategory::App,
                services: None,
                windows: None,
            },
            depth: 0,
            has_children: totals.group_count > 0,
            is_expanded,
            group_size: totals.group_count,
            section: Some(SectionRow {
                id,
                compressed_bytes: totals.compressed_bytes,
            }),
            highlight: None,
            child: None,
            details: false,
            details_expanded: false,
            exited: false,
            rule_below: false,
            absent: false,
            wsl: None,
            lifted: false,
            drop_edge: None,
        }
    }

    fn absent(name: &Arc<str>, pin: &PinnedProcess) -> Self {
        Self {
            row: ProcessRow {
                pid: 0,
                name: name.clone(),
                display_name: if pin.display_name.is_empty() {
                    name.clone()
                } else {
                    pin.display_name.as_str().into()
                },
                cpu_percent: 0.0,
                memory_bytes: 0,
                disk_bytes: 0,
                net_bytes: 0,
                exe_path: pin.exe_path.as_str().into(),
                package_full_name: pin.package_full_name.as_str().into(),
                owner: None,
                owner_pid: None,
                category: ProcessCategory::App,
                services: None,
                windows: None,
            },
            depth: 1,
            has_children: false,
            is_expanded: false,
            group_size: 1,
            section: None,
            highlight: None,
            child: None,
            details: false,
            details_expanded: false,
            exited: false,
            rule_below: false,
            absent: true,
            wsl: None,
            lifted: false,
            drop_edge: None,
        }
    }

    pub(crate) fn height(&self) -> f64 {
        if self.section.is_some() {
            size::SectionRow
        } else {
            size::ProcessRow
        }
    }

    pub(crate) fn stands_for_one_process(&self) -> bool {
        !self.absent
            && self.section.is_none()
            && !matches!(self.wsl, Some(WslRow::Environment { .. }))
            && self.child.as_ref().is_none_or(Child::has_metrics)
            && !(self.has_children && self.is_expanded)
    }
}

pub(crate) fn keep_group_place(
    sections: &mut [Section],
    selected: Option<u32>,
    previous: Option<(u32, usize)>,
) -> Option<(u32, usize)> {
    let pid = selected?;
    let (section, current) = sections.iter().enumerate().find_map(|(at, section)| {
        section
            .groups
            .iter()
            .position(|group| group.members.iter().any(|member| member.pid == pid))
            .map(|group| (at, group))
    })?;

    let groups = &mut sections[section].groups;
    let target = previous
        .filter(|(pinned, _)| *pinned == pid)
        .map_or(current, |(_, at)| at.min(groups.len() - 1));
    if target != current {
        let group = groups.remove(current);
        groups.insert(target, group);
    }
    Some((pid, target))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn heading(id: SectionId) -> DisplayRow {
        DisplayRow::section(id, SectionTotals::default(), true)
    }

    pub(crate) fn process(pid: u32) -> DisplayRow {
        plain_row(&row(pid, "p.exe"), 1)
    }

    fn row(pid: u32, name: &str) -> ProcessRow {
        categorised(pid, name, ProcessCategory::App)
    }

    fn categorised(pid: u32, name: &str, category: ProcessCategory) -> ProcessRow {
        ProcessRow {
            pid,
            name: name.into(),
            display_name: name.into(),
            cpu_percent: pid as f32,
            memory_bytes: 0,
            disk_bytes: 0,
            net_bytes: 0,
            exe_path: "".into(),
            package_full_name: "".into(),
            owner: None,
            owner_pid: None,
            category,
            services: None,
            windows: None,
        }
    }

    fn flat(
        sections: &[Section],
        groups: &HashSet<String>,
        collapsed_sections: &HashSet<SectionId>,
    ) -> Vec<DisplayRow> {
        flatten_for_display(
            sections,
            &ViewState {
                groups,
                processes: &HashSet::new(),
                collapsed_sections,
                exited: &HashSet::new(),
            },
        )
    }

    fn process_pids(rows: &[DisplayRow]) -> Vec<u32> {
        rows.iter()
            .filter(|d| d.section.is_none())
            .map(|d| d.row.pid)
            .collect()
    }

    fn labels(rows: &[DisplayRow]) -> Vec<&str> {
        rows.iter()
            .filter_map(|d| d.section.as_ref().map(|s| s.id.id()))
            .collect()
    }

    fn one_section(groups: Vec<ProcessGroup>) -> Vec<Section> {
        split_by_category(groups)
    }

    fn group(leader: ProcessRow, rest: Vec<ProcessRow>) -> ProcessGroup {
        let mut members = vec![leader.clone()];
        members.extend(rest);
        ProcessGroup { leader, members }
    }

    #[test]
    fn ungrouped_rows_form_singleton_groups() {
        let rows = vec![row(1, "alpha"), row(2, "beta"), row(3, "gamma")];
        let groups = group_by_name(&rows, None);

        assert_eq!(groups.len(), 3);
        assert!(groups.iter().all(|g| g.members.len() == 1));
    }

    #[test]
    fn duplicate_names_group_with_lowest_pid_as_leader_by_default() {
        let rows = vec![
            row(5, "chrome.exe"),
            row(2, "chrome.exe"),
            row(9, "chrome.exe"),
        ];
        let groups = group_by_name(&rows, None);

        assert_eq!(groups.len(), 1, "same-named rows collapse into one group");
        assert_eq!(groups[0].leader.pid, 2, "leader defaults to the lowest pid");
        assert_eq!(groups[0].members.len(), 3);
    }

    #[test]
    fn selecting_a_non_lowest_pid_member_promotes_it_to_leader() {
        let rows = vec![
            row(5, "chrome.exe"),
            row(2, "chrome.exe"),
            row(9, "chrome.exe"),
        ];
        let groups = group_by_name(&rows, Some(9));

        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].leader.pid, 9,
            "selected member becomes the leader, not the lowest pid"
        );
    }

    #[test]
    fn selecting_a_member_does_not_reorder_the_group() {
        let rows = vec![
            row(5, "chrome.exe"),
            row(2, "chrome.exe"),
            row(9, "chrome.exe"),
        ];
        let groups = group_by_name(&rows, Some(9));
        let order: Vec<u32> = groups[0].members.iter().map(|r| r.pid).collect();

        assert_eq!(order, vec![2, 5, 9], "members stay in pid order whoever is selected");
    }

    #[test]
    fn every_service_host_stands_on_its_own() {
        let rows = vec![
            row(5, "svchost.exe"),
            row(2, "svchost.exe"),
            row(7, "chrome.exe"),
            row(9, "chrome.exe"),
        ];
        let groups = group_by_name(&rows, None);
        let sizes: Vec<(u32, usize)> = groups.iter().map(|g| (g.leader.pid, g.members.len())).collect();

        assert_eq!(sizes, vec![(5, 1), (2, 1), (7, 2)]);
    }

    #[test]
    fn leader_sums_metrics_across_the_group() {
        let rows = vec![row(1, "chrome.exe"), row(2, "chrome.exe")];
        let groups = group_by_name(&rows, None);

        assert_eq!(groups[0].leader.cpu_percent, 1.0 + 2.0);
    }

    #[test]
    fn collapsed_groups_render_as_a_single_leader_row() {
        let sections = one_section(vec![group(
            row(2, "chrome.exe"),
            vec![row(5, "chrome.exe"), row(9, "chrome.exe")],
        )]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());

        assert_eq!(process_pids(&out), vec![2]);
        let leader = out.iter().find(|d| d.section.is_none()).unwrap();
        assert!(leader.has_children);
        assert_eq!(leader.group_size, 3);
    }

    #[test]
    fn expanding_a_group_inserts_members_right_after_its_leader() {
        let sections = one_section(vec![
            group(row(10, "alpha"), vec![]),
            group(row(2, "chrome.exe"), vec![row(5, "chrome.exe")]),
            group(row(20, "zeta"), vec![]),
        ]);
        let mut expanded = HashSet::new();
        expanded.insert("chrome.exe".to_string());
        let out = flat(&sections, &expanded, &HashSet::new());

        assert_eq!(
            process_pids(&out),
            vec![10, 2, 2, 5, 20],
            "the heading, then every member including the one it is named after"
        );
        let member = out.iter().find(|d| d.row.pid == 5).unwrap();
        assert_eq!(member.depth, 2, "a group member sits one below its leader");
        assert!(!member.has_children);
    }

    #[test]
    fn an_expanded_group_lists_every_member_under_its_heading() {
        let sections = one_section(vec![group(
            row(2, "AppControl.exe"),
            vec![row(5, "AppControl.exe")],
        )]);
        let mut expanded = HashSet::new();
        expanded.insert("AppControl.exe".to_string());
        let out = flat(&sections, &expanded, &HashSet::new());

        let members: Vec<u32> = out
            .iter()
            .filter(|d| d.stands_for_one_process())
            .map(|d| d.row.pid)
            .collect();
        assert_eq!(members, vec![2, 5]);
    }

    fn group_pids(sections: &[Section]) -> Vec<u32> {
        sections[0].groups.iter().map(|g| g.leader.pid).collect()
    }

    #[test]
    fn a_selected_group_keeps_its_place_among_siblings_across_a_resort() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(2, "beta"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);
        let kept = keep_group_place(&mut sections, Some(2), None);
        assert_eq!(kept, Some((2, 1)), "nothing moves on the first sight");

        let mut resorted = one_section(vec![
            group(row(2, "beta"), vec![]),
            group(row(1, "alpha"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);
        let kept = keep_group_place(&mut resorted, Some(2), kept);

        assert_eq!(kept, Some((2, 1)));
        assert_eq!(group_pids(&resorted), vec![1, 2, 3], "beta pulled back to its place");
    }

    #[test]
    fn a_new_selection_keeps_its_own_place() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(2, "beta"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);

        let kept = keep_group_place(&mut sections, Some(3), Some((1, 0)));

        assert_eq!(kept, Some((3, 2)));
        assert_eq!(group_pids(&sections), vec![1, 2, 3], "the clicked group does not jump");
    }

    #[test]
    fn a_held_group_never_lands_inside_an_expanded_group() {
        let mut sections = one_section(vec![
            group(row(10, "rust-analyzer.exe"), vec![row(11, "rust-analyzer.exe"), row(12, "rust-analyzer.exe")]),
            group(row(20, "proc-macro-srv.exe"), vec![]),
            group(row(30, "cargo.exe"), vec![]),
        ]);
        let kept = keep_group_place(&mut sections, Some(20), None);

        let mut resorted = one_section(vec![
            group(row(30, "cargo.exe"), vec![]),
            group(row(10, "rust-analyzer.exe"), vec![row(11, "rust-analyzer.exe"), row(12, "rust-analyzer.exe")]),
            group(row(20, "proc-macro-srv.exe"), vec![]),
        ]);
        keep_group_place(&mut resorted, Some(20), kept);
        let mut expanded = HashSet::new();
        expanded.insert("rust-analyzer.exe".to_string());
        let out = flat(&resorted, &expanded, &HashSet::new());

        assert_eq!(
            process_pids(&out),
            vec![30, 20, 10, 10, 11, 12],
            "the held group sits between whole groups, never among another group's members"
        );
    }

    #[test]
    fn selecting_a_member_holds_its_whole_group() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(10, "chrome.exe"), vec![row(11, "chrome.exe")]),
        ]);

        assert_eq!(keep_group_place(&mut sections, Some(11), None), Some((11, 1)));
    }

    #[test]
    fn holding_reports_none_when_the_selection_is_not_on_screen() {
        let mut sections = one_section(vec![group(row(1, "alpha"), vec![])]);

        assert_eq!(keep_group_place(&mut sections, Some(99), Some((99, 0))), None);
        assert_eq!(keep_group_place(&mut sections, None, None), None);
    }

    #[test]
    fn categories_render_in_order_under_their_parent_headings() {
        let sections = split_by_category(vec![
            group(categorised(1, "svchost.exe", ProcessCategory::WindowsService), vec![]),
            group(categorised(2, "chrome.exe", ProcessCategory::App), vec![]),
            group(categorised(3, "updater.exe", ProcessCategory::BackgroundThirdParty), vec![]),
            group(categorised(4, "System", ProcessCategory::WindowsKernel), vec![]),
            group(categorised(5, "RuntimeBroker.exe", ProcessCategory::BackgroundMicrosoft), vec![]),
        ]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());

        assert_eq!(
            labels(&out),
            vec![
                "app",
                "background-third-party",
                "background-microsoft",
                "windows-service",
                "windows-kernel",
            ]
        );
        assert_eq!(process_pids(&out), vec![2, 3, 5, 1, 4]);
    }

    #[test]
    fn an_empty_category_gets_no_heading() {
        let sections = split_by_category(vec![group(
            categorised(1, "chrome.exe", ProcessCategory::App),
            vec![],
        )]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());

        assert_eq!(labels(&out), vec!["app"]);
    }

    #[test]
    fn collapsing_a_category_hides_its_processes_but_keeps_the_heading() {
        let sections = split_by_category(vec![
            group(categorised(1, "chrome.exe", ProcessCategory::App), vec![]),
            group(categorised(2, "svchost.exe", ProcessCategory::WindowsService), vec![]),
        ]);
        let mut collapsed = HashSet::new();
        collapsed.insert(SectionId::Category(ProcessCategory::WindowsService));
        let out = flat(&sections, &HashSet::new(), &collapsed);

        assert_eq!(process_pids(&out), vec![1], "the service row is hidden");
        assert!(
            labels(&out).contains(&"windows-service"),
            "its heading stays"
        );
    }

    #[test]
    fn the_kernel_heading_takes_compressed_memory_out_of_its_total() {
        let mut compression = categorised(4, ProcessName::MemoryCompression, ProcessCategory::WindowsKernel);
        compression.memory_bytes = 3_000;
        let mut system = categorised(5, "System", ProcessCategory::WindowsKernel);
        system.memory_bytes = 1_000;

        let sections = split_by_category(vec![group(compression, vec![]), group(system, vec![])]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());
        let heading = out.iter().find(|d| d.section.is_some()).unwrap();

        assert_eq!(heading.row.memory_bytes, 1_000);
        assert_eq!(heading.section.as_ref().unwrap().compressed_bytes, 3_000);
    }

    #[test]
    fn other_sections_report_no_compression() {
        let sections = split_by_category(vec![group(
            categorised(1, "chrome.exe", ProcessCategory::App),
            vec![],
        )]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());
        let heading = out.iter().find(|d| d.section.is_some()).unwrap();

        assert_eq!(heading.section.as_ref().unwrap().compressed_bytes, 0);
    }

    #[test]
    fn a_heading_counts_groups_not_processes() {
        let sections = split_by_category(vec![
            group(
                categorised(1, "chrome.exe", ProcessCategory::App),
                vec![
                    categorised(2, "chrome.exe", ProcessCategory::BackgroundThirdParty),
                    categorised(3, "chrome.exe", ProcessCategory::BackgroundThirdParty),
                ],
            ),
            group(categorised(4, "code.exe", ProcessCategory::App), vec![]),
        ]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());

        let heading = out.iter().find(|d| d.section.is_some()).unwrap();
        assert_eq!(heading.group_size, 2, "two apps, not four processes");
    }

    #[test]
    fn a_group_is_an_app_if_any_member_owns_a_window() {
        let sections = split_by_category(vec![group(
            categorised(1, "chrome.exe", ProcessCategory::BackgroundThirdParty),
            vec![
                categorised(2, "chrome.exe", ProcessCategory::BackgroundThirdParty),
                categorised(3, "chrome.exe", ProcessCategory::App),
            ],
        )]);

        assert_eq!(labels(&flat(&sections, &HashSet::new(), &HashSet::new())), vec!["app"]);
    }

    fn highlighted(rows: &[DisplayRow]) -> Vec<(u8, u32, Highlight)> {
        rows.iter()
            .filter_map(|d| d.highlight.map(|h| (d.depth, d.row.pid, h)))
            .collect()
    }

    fn chrome_expanded(selected: u32) -> Vec<DisplayRow> {
        let rows = vec![
            row(2, "chrome.exe"),
            row(5, "chrome.exe"),
            row(9, "chrome.exe"),
            row(20, "zeta"),
        ];
        let sections = split_by_category(group_by_name(&rows, Some(selected)));
        let mut expanded = HashSet::new();
        expanded.insert("chrome.exe".to_string());
        flat(&sections, &expanded, &HashSet::new())
    }

    #[test]
    fn a_selected_group_highlights_its_heading_and_every_member_as_one_block() {
        let mut out = chrome_expanded(5);
        highlight(&mut out, Some(Selection::Group(5)));

        let top = Highlight { top: true, bottom: false };
        let middle = Highlight { top: false, bottom: false };
        let bottom = Highlight { top: false, bottom: true };
        assert_eq!(
            highlighted(&out),
            vec![(1, 5, top), (2, 2, middle), (2, 5, middle), (2, 9, bottom)]
        );
    }

    #[test]
    fn a_selected_member_is_highlighted_alone() {
        let mut out = chrome_expanded(5);
        highlight(&mut out, Some(Selection::Process(5)));

        assert_eq!(highlighted(&out), vec![(2, 5, Highlight::Whole)]);
    }

    #[test]
    fn a_selected_collapsed_group_is_one_row() {
        let rows = vec![row(2, "chrome.exe"), row(5, "chrome.exe")];
        let sections = split_by_category(group_by_name(&rows, Some(5)));
        let mut out = flat(&sections, &HashSet::new(), &HashSet::new());
        highlight(&mut out, Some(Selection::Group(5)));

        assert_eq!(highlighted(&out), vec![(1, 5, Highlight::Whole)]);
    }

    fn cpu_order() -> Order {
        Order {
            column: ProcessColumn::Cpu,
            descending: true,
        }
    }

    #[test]
    fn cpu_jitter_below_the_shown_precision_does_not_reorder() {
        let mut a = row(1, "alpha");
        a.cpu_percent = 0.01;
        let mut b = row(2, "beta");
        b.cpu_percent = 0.04;
        let mut sections = split_by_category(group_by_name(&[b.clone(), a.clone()], None));
        sort_groups(&mut sections, &cpu_order());
        assert_eq!(group_pids(&sections), vec![1, 2], "both show 0.0%, so by name");

        a.cpu_percent = 0.03;
        b.cpu_percent = 0.02;
        let mut sections = split_by_category(group_by_name(&[a, b], None));
        sort_groups(&mut sections, &cpu_order());
        assert_eq!(group_pids(&sections), vec![1, 2]);
    }

    #[test]
    fn the_kernel_heading_leaves_idle_time_out_of_its_cpu() {
        let mut idle = categorised(0, "System Idle Process", ProcessCategory::WindowsKernel);
        idle.cpu_percent = 52.7;
        let mut system = categorised(4, "System", ProcessCategory::WindowsKernel);
        system.cpu_percent = 1.8;

        let sections = split_by_category(vec![group(idle, vec![]), group(system, vec![])]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());
        let heading = out.iter().find(|d| d.section.is_some()).unwrap();

        assert!((heading.row.cpu_percent - 1.8).abs() < 1e-4, "{}", heading.row.cpu_percent);
    }

    #[test]
    fn idle_time_sorts_below_every_process_by_cpu() {
        let mut idle = categorised(0, "System Idle Process", ProcessCategory::WindowsKernel);
        idle.cpu_percent = 90.0;
        let busy = categorised(4, "System", ProcessCategory::WindowsKernel);
        let mut sections = split_by_category(vec![group(idle, vec![]), group(busy, vec![])]);

        sort_groups(&mut sections, &Order { column: ProcessColumn::Cpu, descending: true });

        assert_eq!(group_pids(&sections), vec![4, 0]);
    }

    #[test]
    fn memory_compression_sorts_below_every_process_by_memory() {
        let mut compression = row(1, ProcessName::MemoryCompression);
        compression.memory_bytes = 3_000;
        let mut small = row(2, "small.exe");
        small.memory_bytes = 0;
        let mut large = row(3, "large.exe");
        large.memory_bytes = 1_000;
        let groups = || group_by_name(&[compression.clone(), small.clone(), large.clone()], None);

        let mut descending = one_section(groups());
        sort_groups(&mut descending, &Order { column: ProcessColumn::Memory, descending: true });
        assert_eq!(group_pids(&descending), vec![3, 2, 1]);

        let mut ascending = one_section(groups());
        sort_groups(&mut ascending, &Order { column: ProcessColumn::Memory, descending: false });
        assert_eq!(group_pids(&ascending), vec![1, 2, 3]);
    }

    #[test]
    fn groups_rank_by_their_summed_value_not_their_busiest_member() {
        let mut worker = row(1, "worker.exe");
        worker.cpu_percent = 3.0;
        let mut other = row(2, "worker.exe");
        other.cpu_percent = 3.0;
        let mut single = row(3, "single.exe");
        single.cpu_percent = 5.0;
        let mut sections = split_by_category(group_by_name(&[single, worker, other], None));
        sort_groups(&mut sections, &cpu_order());

        assert_eq!(group_pids(&sections), vec![1, 3], "6% total beats 5%");
    }

    #[test]
    fn sorting_by_name_follows_the_displayed_name() {
        let mut explorer = row(1, "explorer.exe");
        explorer.display_name = "Windows Explorer".into();
        let mut sections = split_by_category(group_by_name(
            &[explorer, row(2, "Terminal"), row(3, "chrome")],
            None,
        ));
        let order = Order {
            column: ProcessColumn::Name,
            descending: false,
        };
        sort_groups(&mut sections, &order);
        let names: Vec<&str> = sections[0]
            .groups
            .iter()
            .map(|g| &*g.leader.display_name)
            .collect();

        assert_eq!(names, vec!["chrome", "Terminal", "Windows Explorer"]);
    }

    #[test]
    fn ties_break_by_name_ascending_even_when_sorting_descending() {
        let mut sections =
            split_by_category(group_by_name(&[row(0, "zeta"), row(0, "alpha")], None));
        for g in &mut sections[0].groups {
            g.leader.cpu_percent = 0.0;
        }
        sort_groups(&mut sections, &cpu_order());
        let names: Vec<&str> = sections[0].groups.iter().map(|g| &*g.leader.name).collect();

        assert_eq!(names, vec!["alpha", "zeta"]);
    }

    fn hosting(pid: u32, name: &str, services: &[&str]) -> ProcessRow {
        let mut host = row(pid, name);
        host.services = Some(
            services
                .iter()
                .map(|service| HostedService {
                    name: (*service).into(),
                    display_name: (*service).into(),
                })
                .collect(),
        );
        host
    }

    fn with_open(sections: &[Section], pids: &[u32]) -> Vec<DisplayRow> {
        flatten_for_display(
            sections,
            &ViewState {
                groups: &HashSet::new(),
                processes: &pids.iter().copied().collect(),
                collapsed_sections: &HashSet::new(),
                exited: &HashSet::new(),
            },
        )
    }

    fn children(rows: &[DisplayRow]) -> Vec<(u8, u32, &str)> {
        rows.iter()
            .filter_map(|d| {
                let label = match d.child.as_ref()? {
                    Child::Service(service) => &*service.display_name,
                    Child::Window(window) => &*window.title,
                    Child::Console => &*d.row.display_name,
                };
                Some((d.depth, d.row.pid, label))
            })
            .collect()
    }

    fn main_window() -> Option<Arc<[ProcessWindow]>> {
        Some(Arc::from(vec![ProcessWindow {
            handle: 0,
            title: Arc::from("Main window"),
        }]))
    }

    #[test]
    fn an_app_lists_its_windows_before_its_services_even_a_single_one() {
        let mut app = hosting(7, "app.exe", &["Helper"]);
        app.windows = main_window();
        let sections = one_section(vec![group(app, vec![])]);

        let out = with_open(&sections, &[7]);

        assert_eq!(children(&out), vec![(2, 7, "Main window"), (2, 7, "Helper")]);
    }

    #[test]
    fn a_selected_process_with_its_details_open_is_one_block() {
        let mut app = row(7, "app.exe");
        app.windows = main_window();
        let sections = split_by_category(group_by_name(&[app, row(9, "zeta")], None));

        let mut closed = with_open(&sections, &[]);
        highlight(&mut closed, Some(Selection::Process(7)));
        assert_eq!(highlighted(&closed), vec![(1, 7, Highlight::Whole)]);

        let mut open = with_open(&sections, &[7]);
        highlight(&mut open, Some(Selection::Process(7)));
        assert_eq!(
            highlighted(&open),
            vec![
                (1, 7, Highlight { top: true, bottom: false }),
                (2, 7, Highlight { top: false, bottom: true }),
            ]
        );
    }

    #[test]
    fn a_process_hosting_services_can_open_to_show_them() {
        let sections = one_section(vec![group(hosting(7, "svchost.exe", &["Audio", "Power"]), vec![])]);

        let closed = with_open(&sections, &[]);
        let host = closed.iter().find(|d| d.row.pid == 7).unwrap();
        assert!(host.details && !host.details_expanded);
        assert!(children(&closed).is_empty());

        let open = with_open(&sections, &[7]);
        assert_eq!(children(&open), vec![(2, 7, "Audio"), (2, 7, "Power")]);
    }

    #[test]
    fn a_process_without_services_has_nothing_to_open() {
        let sections = one_section(vec![group(row(7, "app.exe"), vec![])]);
        let out = with_open(&sections, &[7]);

        assert!(!out.iter().any(|d| d.details));
        assert!(children(&out).is_empty());
    }

    #[test]
    fn a_member_of_an_open_group_shows_its_services_one_level_deeper() {
        let sections = one_section(vec![group(
            hosting(2, "svc.exe", &["A"]),
            vec![hosting(5, "svc.exe", &["B"])],
        )]);
        let out = flatten_for_display(
            &sections,
            &ViewState {
                groups: &["svc.exe".to_string()].into_iter().collect(),
                processes: &[5].into_iter().collect(),
                collapsed_sections: &HashSet::new(),
                exited: &HashSet::new(),
            },
        );

        assert_eq!(children(&out), vec![(3, 5, "B")]);
        let heading = out.iter().find(|d| d.depth == 1 && d.section.is_none()).unwrap();
        assert!(!heading.details, "a group heading stands for several processes, not one");
    }

    #[test]
    fn a_service_row_is_painted_as_the_tail_of_its_hosts_block() {
        let sections = one_section(vec![group(hosting(7, "svchost.exe", &["Audio"]), vec![])]);
        let mut out = with_open(&sections, &[7]);
        highlight(&mut out, Some(Selection::Process(7)));

        assert_eq!(
            highlighted(&out),
            vec![
                (1, 7, Highlight { top: true, bottom: false }),
                (2, 7, Highlight { top: false, bottom: true }),
            ]
        );
    }

    fn console_of(pid: u32, owner: u32) -> ProcessRow {
        let mut console = row(pid, "conhost.exe");
        console.display_name = "Console Window Host".into();
        console.owner_pid = Some(owner);
        console
    }

    static DEFAULT_ORDER: std::sync::LazyLock<SectionOrder> = std::sync::LazyLock::new(SectionOrder::default);

    fn grouped<'a>(by_type: bool, order: &'a Order, pins: &'a Pins) -> Grouping<'a> {
        Grouping {
            selected: None,
            kept: None,
            order,
            by_type,
            pins,
            wsl: &[],
            section_order: &DEFAULT_ORDER,
        }
    }

    fn pinned_sections_for(rows: &[ProcessRow], by_type: bool, pins: &[&str]) -> Vec<Section> {
        let pins: Pins = pins
            .iter()
            .map(|name| (Arc::from(*name), PinnedProcess::default()))
            .collect();
        let mut cache = GroupsCache::empty();
        cache.get(rows, &[], grouped(by_type, &cpu_order(), &pins));
        cache.sections
    }

    const APPS: SectionId = SectionId::Category(ProcessCategory::App);
    const KERNEL: SectionId = SectionId::Category(ProcessCategory::WindowsKernel);

    #[test]
    fn sections_come_in_the_kept_order() {
        let order = SectionOrder::default().moved(KERNEL, Some(APPS));
        let rows = [row(1, "a.exe"), categorised(2, "System", ProcessCategory::WindowsKernel)];
        let ids: Vec<SectionId> = split_keeping(group_by_name(&rows, None), &Pins::new(), &[], None, &order)
            .iter()
            .map(|section| section.id)
            .collect();
        assert_eq!(ids, [KERNEL, APPS]);
    }

    #[test]
    fn a_moved_section_lands_before_the_one_named_or_last() {
        let last = SectionOrder::default().moved(APPS, None);
        assert_eq!(last.ids().last(), Some(&APPS));
        assert_eq!(last.ids().len(), SectionOrder::default().ids().len());
        let third_party = SectionId::Category(ProcessCategory::BackgroundThirdParty);
        assert_eq!(last.moved(APPS, Some(third_party)), SectionOrder::default());
    }

    #[test]
    fn kept_ranks_order_the_sections_and_the_unranked_follow_in_their_usual_order() {
        let order = SectionOrder::ranked(|id| (id == KERNEL).then_some(0));
        assert_eq!(order.ids()[..2], [KERNEL, SectionId::Pinned]);
    }

    fn sections_for(rows: &[ProcessRow]) -> Vec<Section> {
        pinned_sections_for(rows, true, &[])
    }

    fn untyped_sections_for(rows: &[ProcessRow]) -> Vec<Section> {
        pinned_sections_for(rows, false, &[])
    }

    #[test]
    fn without_grouping_by_type_there_are_no_section_rows() {
        let rows = vec![
            row(10, "notepad.exe"),
            categorised(20, "zeta", ProcessCategory::BackgroundThirdParty),
        ];

        let typed = with_open(&sections_for(&rows), &[]);
        assert_eq!(typed.iter().filter(|d| d.section.is_some()).count(), 2);

        let flat = with_open(&untyped_sections_for(&rows), &[]);
        assert!(flat.iter().all(|d| d.section.is_none()));
        let mut pids = process_pids(&flat);
        pids.sort_unstable();
        assert_eq!(pids, vec![10, 20]);
        assert!(flat.iter().all(|d| d.depth == 1));
    }

    #[test]
    fn without_grouping_by_type_one_order_runs_across_every_type() {
        let mut app = row(10, "notepad.exe");
        app.cpu_percent = 1.0;
        let mut background = categorised(20, "zeta", ProcessCategory::BackgroundThirdParty);
        background.cpu_percent = 9.0;

        let flat = with_open(&untyped_sections_for(&[app, background]), &[]);
        assert_eq!(process_pids(&flat), vec![20, 10]);
    }

    #[test]
    fn same_name_groups_survive_without_grouping_by_type() {
        let rows = vec![row(10, "notepad.exe"), row(11, "notepad.exe"), row(20, "zeta")];
        let flat = with_open(&untyped_sections_for(&rows), &[]);

        let group = flat.iter().find(|d| d.has_children).unwrap();
        assert_eq!(group.group_size, 2);
    }

    #[test]
    fn a_console_host_moves_under_the_process_it_serves() {
        let rows = vec![row(10, "cargo.exe"), console_of(11, 10), row(20, "zeta")];
        let sections = sections_for(&rows);

        let closed = with_open(&sections, &[]);
        assert!(!process_pids(&closed).contains(&11), "not a top-level row any more");
        assert!(closed.iter().find(|d| d.row.pid == 10).unwrap().details);

        let open = with_open(&sections, &[10]);
        let console = open.iter().find(|d| d.row.pid == 11).unwrap();
        assert_eq!(console.child, Some(Child::Console));
        assert_eq!(console.depth, 2);
    }

    #[test]
    fn a_console_host_whose_owner_is_gone_stays_on_its_own() {
        let rows = vec![console_of(11, 999), row(20, "zeta")];
        let out = with_open(&sections_for(&rows), &[]);

        assert!(process_pids(&out).contains(&11));
    }

    #[test]
    fn a_console_host_counts_towards_its_owners_section_and_can_be_selected() {
        let mut owner = row(10, "cargo.exe");
        owner.memory_bytes = 100;
        let mut console = console_of(11, 10);
        console.memory_bytes = 7;
        let sections = sections_for(&[owner, console]);

        let mut out = with_open(&sections, &[10]);
        let heading = out.iter().find(|d| d.section.is_some()).unwrap();
        assert_eq!(heading.row.memory_bytes, 107);

        highlight(&mut out, Some(Selection::Process(11)));
        assert_eq!(highlighted(&out), vec![(2, 11, Highlight::Whole)]);
    }

    fn ruled(rows: &[DisplayRow]) -> Vec<u32> {
        rows.iter().filter(|d| d.rule_below).map(|d| d.row.pid).collect()
    }

    #[test]
    fn by_type_pinned_groups_get_their_own_section_first() {
        let rows = vec![
            row(10, "notepad.exe"),
            categorised(20, "zeta", ProcessCategory::BackgroundThirdParty),
            categorised(30, "agent.exe", ProcessCategory::BackgroundThirdParty),
        ];

        let out = with_open(&pinned_sections_for(&rows, true, &["agent.exe"]), &[]);

        assert_eq!(labels(&out), vec!["pinned", "app", "background-third-party"]);
        assert_eq!(process_pids(&out), vec![30, 10, 20]);
        assert_eq!(ruled(&out), Vec::<u32>::new(), "headings part the sections already");
    }

    #[test]
    fn without_grouping_by_type_pinned_groups_come_first_above_a_rule() {
        let mut busy = row(10, "notepad.exe");
        busy.cpu_percent = 50.0;
        let rows = vec![busy, row(20, "zeta"), row(30, "agent.exe"), row(31, "agent.exe")];

        let out = with_open(&untyped_pinned(&rows, &["agent.exe"]), &[]);
        assert!(out.iter().all(|d| d.section.is_none()));
        assert_eq!(process_pids(&out), vec![30, 10, 20]);
        assert_eq!(ruled(&out), vec![30]);

        let mut expanded = HashSet::new();
        expanded.insert("agent.exe".to_string());
        let open = flat(&untyped_pinned(&rows, &["agent.exe"]), &expanded, &HashSet::new());
        assert_eq!(process_pids(&open), vec![30, 31, 30, 10, 20]);
        let under: Vec<usize> = open
            .iter()
            .enumerate()
            .filter(|(_, d)| d.rule_below)
            .map(|(at, _)| at)
            .collect();
        assert_eq!(under, vec![2], "the rule runs under the last member");
    }

    fn untyped_pinned(rows: &[ProcessRow], pins: &[&str]) -> Vec<Section> {
        pinned_sections_for(rows, false, pins)
    }

    #[test]
    fn a_rule_needs_something_on_both_sides() {
        let rows = vec![row(10, "notepad.exe"), row(20, "zeta")];

        assert_eq!(ruled(&with_open(&untyped_pinned(&rows, &[]), &[])), Vec::<u32>::new());
        assert_eq!(
            ruled(&with_open(&untyped_pinned(&rows, &["notepad.exe", "zeta"]), &[])),
            Vec::<u32>::new()
        );
    }

    fn absent(rows: &[DisplayRow]) -> Vec<&str> {
        rows.iter().filter(|d| d.absent).map(|d| &*d.row.name).collect()
    }

    #[test]
    fn a_pin_that_is_not_running_stays_in_the_pinned_section() {
        let rows = vec![row(10, "notepad.exe"), row(30, "agent.exe")];

        let out = with_open(&pinned_sections_for(&rows, true, &["agent.exe", "Zed.exe", "cargo.exe"]), &[]);

        assert_eq!(labels(&out), vec!["pinned", "app"]);
        assert_eq!(absent(&out), vec!["cargo.exe", "Zed.exe"]);
        let heading = out.iter().find(|d| d.section.is_some()).unwrap();
        assert_eq!(heading.group_size, 3, "the heading counts what is not running too");
        let pinned: Vec<&str> = out.iter().skip(1).take(3).map(|d| &*d.row.name).collect();
        assert_eq!(pinned, vec!["agent.exe", "cargo.exe", "Zed.exe"], "running first");
    }

    #[test]
    fn a_pin_that_is_not_running_keeps_the_path_it_was_pinned_with() {
        let pin = PinnedProcess {
            exe_path: r"C:\tools\agent.exe".into(),
            package_full_name: String::new(),
            display_name: "Build Agent".into(),
        };
        let pins: Pins = [(Arc::from("agent.exe"), pin)].into_iter().collect();
        let order = cpu_order();
        let mut cache = GroupsCache::empty();
        cache.get(&[row(10, "notepad.exe")], &[], grouped(true, &order, &pins));

        let out = with_open(&cache.sections, &[]);
        let placeholder = out.iter().find(|d| d.absent).unwrap();

        assert_eq!(&*placeholder.row.exe_path, r"C:\tools\agent.exe");
        assert_eq!(&*placeholder.row.display_name, "Build Agent", "it reads as it did while running");
        assert_eq!(&*placeholder.row.name, "agent.exe");
    }

    #[test]
    fn without_grouping_by_type_a_pin_that_is_not_running_sits_above_the_rule() {
        let rows = vec![row(10, "notepad.exe")];

        let out = with_open(&untyped_pinned(&rows, &["agent.exe"]), &[]);

        assert_eq!(absent(&out), vec!["agent.exe"]);
        assert!(out[0].absent && out[0].rule_below, "{:?}", ruled(&out));
        assert_eq!(out[1].row.pid, 10);
    }

    #[test]
    fn a_pin_that_is_not_running_is_never_part_of_a_selection() {
        let rows = vec![row(10, "notepad.exe")];
        let mut out = with_open(&pinned_sections_for(&rows, true, &["agent.exe"]), &[]);
        assert!(!out.iter().find(|d| d.absent).unwrap().stands_for_one_process());

        highlight(&mut out, Some(Selection::Process(0)));
        highlight(&mut out, Some(Selection::Group(0)));

        assert!(out.iter().all(|d| d.highlight.is_none()));
    }

    #[test]
    fn a_pinned_section_collapses_like_any_other() {
        let rows = vec![row(10, "notepad.exe"), row(30, "agent.exe")];
        let sections = pinned_sections_for(&rows, true, &["agent.exe"]);

        let collapsed: HashSet<SectionId> = [SectionId::Pinned].into_iter().collect();
        let out = flat(&sections, &HashSet::new(), &collapsed);

        assert_eq!(labels(&out), vec!["pinned", "app"]);
        assert_eq!(process_pids(&out), vec![10]);
        assert_eq!(SectionId::from_id(SectionId::Pinned.id()), Some(SectionId::Pinned));
    }

    #[test]
    fn a_selected_group_follows_its_pin_both_ways() {
        let rows = vec![row(10, "notepad.exe"), row(30, "agent.exe")];
        let mut screen = Screen::new();
        let selected = Some(Selection::Group(30));
        screen.show(&rows, selected);

        screen.pins.insert("agent.exe".into(), PinnedProcess::default());
        let pinned = screen.show(&rows, selected);
        assert_eq!(labels(&pinned), vec!["pinned", "app"]);
        assert_eq!(process_pids(&pinned), vec![30, 10]);

        screen.pins.clear();
        let unpinned = screen.show(&rows, selected);
        assert_eq!(labels(&unpinned), vec!["app"]);
    }

    struct Screen {
        cache: GroupsCache,
        held: Held,
        pins: Pins,
    }

    impl Screen {
        fn new() -> Self {
            Self {
                cache: GroupsCache::empty(),
                held: Held::default(),
                pins: Pins::new(),
            }
        }

        fn show(&mut self, rows: &[ProcessRow], selection: Option<Selection>) -> Vec<DisplayRow> {
            let pid = selection.and_then(|s| match s {
                Selection::Process(pid) | Selection::Group(pid) => Some(pid),
                Selection::Linux(_) => None,
            });
            let exited_rows = self.held.exited(selection, rows);
            let kept = self.held.section(selection);
            let exited: HashSet<u32> = exited_rows.iter().map(|r| r.pid).collect();
            let order = cpu_order();
            let sections = self.cache.get(
                rows,
                &exited_rows,
                Grouping {
                    selected: pid,
                    kept,
                    order: &order,
                    by_type: true,
                    pins: &self.pins,
                    wsl: &[],
                    section_order: &DEFAULT_ORDER,
                },
            );
            self.held.hold(sections, selection);
            let mut out = flatten_for_display(
                sections,
                &ViewState {
                    groups: &["notepad.exe".to_string()].into_iter().collect(),
                    processes: &HashSet::new(),
                    collapsed_sections: &HashSet::new(),
                    exited: &exited,
                },
            );
            highlight(&mut out, selection);
            out
        }
    }

    fn exited_pids(rows: &[DisplayRow]) -> Vec<u32> {
        rows.iter().filter(|d| d.exited).map(|d| d.row.pid).collect()
    }

    #[test]
    fn a_selected_process_that_exits_stays_on_screen_marked_exited() {
        let mut screen = Screen::new();
        let mut notepad = row(7, "notepad.exe");
        notepad.memory_bytes = 5_000;
        screen.show(&[notepad, row(9, "zeta")], Some(Selection::Process(7)));

        let after = screen.show(&[row(9, "zeta")], Some(Selection::Process(7)));

        let ghost = after.iter().find(|d| d.row.pid == 7).expect("still on screen");
        assert!(ghost.exited);
        assert_eq!(ghost.row.memory_bytes, 0, "no numbers for a process that is gone");
        assert_eq!(ghost.highlight, Some(Highlight::Whole), "and still selected");
    }

    #[test]
    fn an_exited_process_stays_across_further_reports_while_selected() {
        let mut screen = Screen::new();
        screen.show(&[row(7, "notepad.exe"), row(9, "zeta")], Some(Selection::Process(7)));
        screen.show(&[row(9, "zeta")], Some(Selection::Process(7)));

        let later = screen.show(&[row(9, "zeta"), row(11, "new.exe")], Some(Selection::Process(7)));

        assert_eq!(exited_pids(&later), vec![7]);
    }

    #[test]
    fn selecting_something_else_lets_the_exited_process_go() {
        let mut screen = Screen::new();
        screen.show(&[row(7, "notepad.exe"), row(9, "zeta")], Some(Selection::Process(7)));
        screen.show(&[row(9, "zeta")], Some(Selection::Process(7)));

        let after = screen.show(&[row(9, "zeta")], Some(Selection::Process(9)));

        assert!(!process_pids(&after).contains(&7));
    }

    #[test]
    fn a_member_of_a_selected_group_that_exits_stays_inside_the_group() {
        let mut screen = Screen::new();
        let group = [row(2, "notepad.exe"), row(5, "notepad.exe"), row(9, "zeta")];
        screen.show(&group, Some(Selection::Group(2)));

        let after = screen.show(&[row(2, "notepad.exe"), row(9, "zeta")], Some(Selection::Group(2)));

        assert_eq!(exited_pids(&after), vec![5]);
        let members: Vec<u32> = after.iter().filter(|d| d.depth == 2).map(|d| d.row.pid).collect();
        assert_eq!(members, vec![2, 5], "the exited member keeps its place among the members");
        let heading = after.iter().find(|d| d.has_children && d.section.is_none()).unwrap();
        assert!(!heading.exited, "one member is still alive");
    }

    #[test]
    fn a_selected_group_whose_members_all_exit_stays_as_an_exited_group() {
        let mut screen = Screen::new();
        screen.show(
            &[row(2, "notepad.exe"), row(5, "notepad.exe"), row(9, "zeta")],
            Some(Selection::Group(2)),
        );

        let after = screen.show(&[row(9, "zeta")], Some(Selection::Group(2)));

        let heading = after
            .iter()
            .find(|d| d.has_children && d.section.is_none())
            .expect("the group is still there");
        assert!(heading.exited);
        assert_eq!(exited_pids(&after), vec![2, 2, 5]);
    }

    #[test]
    fn a_heading_can_never_be_selected() {
        let sections = split_by_category(vec![group(
            categorised(1, "chrome.exe", ProcessCategory::App),
            vec![],
        )]);
        let out = flat(&sections, &HashSet::new(), &HashSet::new());

        assert!(out.iter().filter(|d| d.section.is_some()).all(|d| d.row.pid == 0));
    }

    fn linux(global_pid: u32, local_pid: u32, name: &str) -> WslProcess {
        WslProcess {
            global_pid,
            row: categorised(local_pid, name, ProcessCategory::Wsl),
        }
    }

    const UBUNTU: u64 = 11;
    const WEB: u64 = 33;

    fn environment(pid_ns: u64, name: &str, kind: EnvironmentKind, processes: Vec<WslProcess>) -> WslEnvironment {
        WslEnvironment {
            pid_ns,
            name: name.into(),
            kind,
            processes: Arc::from(processes),
        }
    }

    fn ubuntu_and_web() -> Vec<WslEnvironment> {
        vec![
            environment(
                UBUNTU,
                "Ubuntu",
                EnvironmentKind::CurrentDistro,
                vec![linux(100, 1, "init"), linux(101, 2, "bash")],
            ),
            environment(WEB, "web", EnvironmentKind::DockerContainer, vec![linux(300, 1, "nginx")]),
        ]
    }

    fn vm(pid: u32) -> ProcessRow {
        let mut vm = categorised(pid, "vmmemWSL", ProcessCategory::WindowsKernel);
        vm.memory_bytes = 5_000;
        vm
    }

    fn wsl_sections_for(rows: &[ProcessRow], by_type: bool, wsl: &[WslEnvironment]) -> Vec<Section> {
        let pins = Pins::new();
        let order = cpu_order();
        let mut cache = GroupsCache::empty();
        cache.get(rows, &[], Grouping { wsl, ..grouped(by_type, &order, &pins) });
        cache.sections
    }

    fn opened(sections: &[Section], open: &[u64]) -> Vec<DisplayRow> {
        let groups: HashSet<String> = open.iter().map(|pid_ns| environment_key(*pid_ns)).collect();
        flat(sections, &groups, &HashSet::new())
    }

    fn names(rows: &[DisplayRow]) -> Vec<(u8, &str)> {
        rows.iter()
            .filter(|d| d.section.is_none())
            .map(|d| (d.depth, &*d.row.name))
            .collect()
    }

    #[test]
    fn the_wsl_vm_heads_its_own_section_of_environments() {
        let rows = vec![row(10, "notepad.exe"), vm(20)];

        let out = opened(&wsl_sections_for(&rows, true, &ubuntu_and_web()), &[]);

        assert_eq!(labels(&out), vec!["app", "wsl"]);
        assert_eq!(names(&out), vec![(1, "notepad.exe"), (1, "Ubuntu"), (1, "web")]);
        let heading = out.iter().find(|d| d.section.as_ref().is_some_and(|s| s.id.id() == "wsl")).unwrap();
        assert_eq!(heading.row.memory_bytes, 5_000, "what the VM costs Windows, not a sum of Linux rows");
        assert_eq!(heading.row.cpu_percent, 20.0);
        assert_eq!(heading.group_size, 2);
    }

    #[test]
    fn an_open_environment_lists_its_processes_one_level_down_and_no_deeper() {
        let rows = vec![vm(20)];

        let out = opened(&wsl_sections_for(&rows, true, &ubuntu_and_web()), &[UBUNTU]);

        assert_eq!(
            names(&out),
            vec![(1, "Ubuntu"), (2, "bash"), (2, "init"), (1, "web")],
            "processes sorted like everything else, all at the depth of a group member"
        );
        let ubuntu = out.iter().find(|d| &*d.row.name == "Ubuntu").unwrap();
        assert!(ubuntu.has_children && ubuntu.is_expanded);
        assert_eq!(ubuntu.group_size, 2);
        assert_eq!(ubuntu.row.cpu_percent, 3.0, "an environment sums its processes");
        assert!(!ubuntu.stands_for_one_process());
    }

    #[test]
    fn environments_nothing_names_open_one_at_a_time() {
        let unnamed = |pid_ns, global_pid, name| {
            environment(pid_ns, "", EnvironmentKind::Unknown, vec![linux(global_pid, 1, name)])
        };
        let sections = wsl_sections_for(&[vm(20)], true, &[unnamed(7, 700, "one"), unnamed(8, 800, "two")]);

        let out = opened(&sections, &[7]);

        let open: Vec<&str> = out.iter().filter(|d| d.depth == 2).map(|d| &*d.row.name).collect();
        assert_eq!(open, vec!["one"]);
    }

    #[test]
    fn a_linux_process_is_told_apart_from_a_windows_one_with_the_same_pid() {
        let rows = vec![categorised(1, "svchost.exe", ProcessCategory::WindowsService), vm(20)];
        let sections = wsl_sections_for(&rows, true, &ubuntu_and_web());

        let mut windows = opened(&sections, &[UBUNTU]);
        highlight(&mut windows, Some(Selection::Process(1)));
        assert_eq!(highlighted(&windows), vec![(1, 1, Highlight::Whole)]);
        assert!(windows.iter().filter(|d| d.highlight.is_some()).all(|d| d.wsl.is_none()));

        let mut linux = opened(&sections, &[UBUNTU]);
        highlight(&mut linux, Some(Selection::Linux(100)));
        let lit: Vec<&DisplayRow> = linux.iter().filter(|d| d.highlight.is_some()).collect();
        assert_eq!(lit.len(), 1);
        assert_eq!(lit[0].wsl, Some(WslRow::Process { global_pid: 100 }));
        assert_eq!(lit[0].row.pid, 1, "the row shows the pid its own namespace sees");
    }

    #[test]
    fn an_environment_heading_is_never_part_of_a_group_selection() {
        let sections = wsl_sections_for(&[vm(20)], true, &ubuntu_and_web());
        let mut out = opened(&sections, &[UBUNTU]);

        highlight(&mut out, Some(Selection::Group(0)));

        assert!(out.iter().all(|d| d.highlight.is_none()));
    }

    #[test]
    fn without_an_agent_the_vm_still_heads_an_empty_wsl_section() {
        let out = opened(&wsl_sections_for(&[row(10, "notepad.exe"), vm(20)], true, &[]), &[]);

        assert_eq!(labels(&out), vec!["app", "wsl"]);
        assert_eq!(names(&out), vec![(1, "notepad.exe")]);
        let heading = out.last().unwrap();
        assert!(!heading.has_children);
    }

    #[test]
    fn without_the_vm_and_its_agent_there_is_no_wsl_section() {
        let out = opened(&wsl_sections_for(&[row(10, "notepad.exe")], true, &[]), &[]);

        assert_eq!(labels(&out), vec!["app"]);
    }

    #[test]
    fn without_grouping_by_type_the_vm_opens_to_its_environments() {
        let rows = vec![row(10, "notepad.exe"), vm(20)];
        let sections = wsl_sections_for(&rows, false, &ubuntu_and_web());

        let closed = opened(&sections, &[]);
        assert!(closed.iter().all(|d| d.section.is_none()));
        assert_eq!(names(&closed), vec![(1, "vmmemWSL"), (1, "notepad.exe")]);
        let vm_row = &closed[0];
        assert!(vm_row.details && !vm_row.details_expanded, "the VM carries a chevron");
        assert!(vm_row.stands_for_one_process(), "and is still selected as the process it is");

        let groups: HashSet<String> = [environment_key(UBUNTU)].into_iter().collect();
        let open = flatten_for_display(
            &sections,
            &ViewState {
                groups: &groups,
                processes: &[20].into_iter().collect(),
                collapsed_sections: &HashSet::new(),
                exited: &HashSet::new(),
            },
        );
        assert_eq!(
            names(&open),
            vec![(1, "vmmemWSL"), (1, "Ubuntu"), (2, "bash"), (2, "init"), (1, "web"), (1, "notepad.exe")],
            "the environments sit level with the VM, as they do under the section heading"
        );

        let mut selected = open.clone();
        highlight(&mut selected, Some(Selection::Process(20)));
        assert_eq!(
            selected.iter().filter(|d| d.highlight.is_some()).count(),
            1,
            "selecting the VM does not take its environments with it"
        );
    }

    #[test]
    fn without_grouping_by_type_or_the_vm_environments_follow_the_windows_processes() {
        let out = opened(&wsl_sections_for(&[row(10, "notepad.exe")], false, &ubuntu_and_web()), &[]);

        assert_eq!(names(&out), vec![(1, "notepad.exe"), (1, "Ubuntu"), (1, "web")]);
    }

    #[test]
    fn an_environment_collapses_and_its_section_persists_under_its_own_id() {
        let sections = wsl_sections_for(&[vm(20)], true, &ubuntu_and_web());
        let collapsed: HashSet<SectionId> = [SectionId::Category(ProcessCategory::Wsl)].into_iter().collect();

        let out = flat(&sections, &HashSet::new(), &collapsed);

        assert_eq!(labels(&out), vec!["wsl"]);
        assert!(names(&out).is_empty());
        assert_eq!(
            SectionId::from_id("wsl"),
            Some(SectionId::Category(ProcessCategory::Wsl))
        );
    }
}
