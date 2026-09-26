use app_contracts::features::processes::{
    HostedService, ProcessCategory, ProcessColumn, ProcessRow,
};

use crate::widgets::table_cell::Highlight;

pub(crate) struct ProcessName;

#[expect(non_upper_case_globals)]
impl ProcessName {
    pub(crate) const MemoryCompression: &str = "Memory Compression";
    const ServiceHost: &str = "svchost.exe";
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

pub(crate) struct Section {
    pub(crate) category: ProcessCategory,
    groups: Vec<ProcessGroup>,
    consoles: HashMap<u32, Vec<ProcessRow>>,
}

#[cfg(test)]
pub(crate) fn split_by_category(groups: Vec<ProcessGroup>) -> Vec<Section> {
    split_keeping(groups, None)
}

fn split_keeping(groups: Vec<ProcessGroup>, keep: Option<(u32, ProcessCategory)>) -> Vec<Section> {
    let mut by_category: HashMap<ProcessCategory, Vec<ProcessGroup>> = HashMap::new();
    for group in groups {
        let category = match keep {
            Some((pid, kept)) if group.members.iter().any(|member| member.pid == pid) => kept,
            _ => group_category(&group),
        };
        by_category.entry(category).or_default().push(group);
    }

    ProcessCategory::ORDER
        .into_iter()
        .filter_map(|category| {
            let groups = by_category.remove(&category)?;
            (!groups.is_empty()).then_some(Section {
                category,
                groups,
                consoles: HashMap::new(),
            })
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
        }
        for console in section.consoles.values().flatten() {
            totals.add(console);
        }
        totals
    }
}

pub(crate) struct ViewState<'a> {
    pub(crate) groups: &'a HashSet<String>,
    pub(crate) processes: &'a HashSet<u32>,
    pub(crate) collapsed_sections: &'a HashSet<ProcessCategory>,
    pub(crate) exited: &'a HashSet<u32>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Child {
    Window(Arc<str>),
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

fn process_row(row: &ProcessRow, depth: u8, expanded: &ViewState<'_>, section: &Section) -> DisplayRow {
    let details = children_of(row).next().is_some() || section.consoles.contains_key(&row.pid);
    DisplayRow {
        exited: expanded.exited.contains(&row.pid),
        row: row.clone(),
        depth,
        has_children: false,
        is_expanded: false,
        group_size: 1,
        section: None,
        highlight: None,
        child: None,
        details,
        details_expanded: details && expanded.processes.contains(&row.pid),
    }
}

fn child_row(row: &ProcessRow, depth: u8, child: Child) -> DisplayRow {
    DisplayRow {
        row: row.clone(),
        depth,
        has_children: false,
        is_expanded: false,
        group_size: 1,
        section: None,
        highlight: None,
        child: Some(child),
        details: false,
        details_expanded: false,
        exited: false,
    }
}

fn push_with_details(out: &mut Vec<DisplayRow>, host: DisplayRow, section: &Section) {
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
}

pub(crate) fn flatten_for_display(sections: &[Section], expanded: &ViewState<'_>) -> Vec<DisplayRow> {
    let mut out = Vec::new();

    for section in sections {
        let totals = SectionTotals::of(section);
        let section_expanded = !expanded.collapsed_sections.contains(&section.category);
        out.push(DisplayRow::section(
            section.category,
            totals,
            section_expanded,
        ));

        if !section_expanded {
            continue;
        }

        for group in &section.groups {
            if group.members.len() == 1 {
                push_with_details(&mut out, process_row(&group.leader, 1, expanded, section), section);
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
                    push_with_details(&mut out, process_row(member, 2, expanded, section), section);
                }
            }
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

fn compare_by(column: ProcessColumn, a: &ProcessRow, b: &ProcessRow) -> Ordering {
    match column {
        ProcessColumn::Name => a
            .display_name
            .chars()
            .flat_map(char::to_lowercase)
            .cmp(b.display_name.chars().flat_map(char::to_lowercase)),
        ProcessColumn::Cpu => tenths(a.cpu_percent).cmp(&tenths(b.cpu_percent)),
        ProcessColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
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
    }
}

pub(crate) struct GroupsCache {
    rows_ptr: *const ProcessRow,
    rows_len: usize,
    exited: Vec<u32>,
    selected: Option<u32>,
    kept: Option<ProcessCategory>,
    order: Order,
    sections: Vec<Section>,
}

impl GroupsCache {
    pub(crate) fn empty() -> Self {
        Self {
            rows_ptr: std::ptr::null(),
            rows_len: 0,
            exited: Vec::new(),
            selected: None,
            kept: None,
            order: Order::default(),
            sections: Vec::new(),
        }
    }

    pub(crate) fn get(
        &mut self,
        rows: &[ProcessRow],
        exited: &[ProcessRow],
        selected: Option<u32>,
        kept: Option<ProcessCategory>,
        order: &Order,
    ) -> &mut [Section] {
        let rows_ptr = rows.as_ptr();
        let same_exited = self.exited.len() == exited.len()
            && self.exited.iter().zip(exited).all(|(pid, row)| *pid == row.pid);
        if self.rows_ptr != rows_ptr
            || self.rows_len != rows.len()
            || !same_exited
            || self.selected != selected
            || self.kept != kept
            || self.order != *order
        {
            let (mut rest, consoles) = detach_consoles(rows);
            rest.extend(exited.iter().cloned());
            self.sections = split_keeping(group_by_name(&rest, selected), selected.zip(kept));
            attach_consoles(&mut self.sections, consoles);
            sort_groups(&mut self.sections, order);
            self.rows_ptr = rows_ptr;
            self.rows_len = rows.len();
            self.exited = exited.iter().map(|row| row.pid).collect();
            self.selected = selected;
            self.kept = kept;
            self.order = *order;
        }
        &mut self.sections
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct SectionRow {
    pub(crate) category: ProcessCategory,
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Selection {
    Process(u32),
    Group(u32),
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
    section: Option<ProcessCategory>,
}

fn section_of(sections: &[Section], selection: Option<Selection>) -> Option<ProcessCategory> {
    let pid = match selection? {
        Selection::Process(pid) | Selection::Group(pid) => pid,
    };
    sections
        .iter()
        .find(|section| {
            section
                .groups
                .iter()
                .any(|group| group.members.iter().any(|member| member.pid == pid))
        })
        .map(|section| section.category)
}

impl Held {
    pub(crate) fn section(&self, selection: Option<Selection>) -> Option<ProcessCategory> {
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

pub(crate) fn highlight(rows: &mut [DisplayRow], selection: Option<Selection>) {
    let Some(selection) = selection else {
        return;
    };
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
        match selection {
            Selection::Group(pid) if span[0].section.is_none() && span[0].row.pid == pid => {
                let last = span.len() - 1;
                for (i, d) in span.iter_mut().enumerate() {
                    d.highlight = Some(Highlight {
                        top: i == 0,
                        bottom: i == last,
                    });
                }
            }
            Selection::Process(pid) => {
                for d in span.iter_mut() {
                    if d.stands_for_one_process() && d.row.pid == pid {
                        d.highlight = Some(Highlight::Whole);
                    }
                }
            }
            Selection::Group(_) => {}
        }
        at += members + 1;
    }
}

impl DisplayRow {
    fn section(category: ProcessCategory, totals: SectionTotals, is_expanded: bool) -> Self {
        let label = category.id();
        Self {
            row: ProcessRow {
                pid: 0,
                name: label.into(),
                display_name: label.into(),
                cpu_percent: totals.cpu_percent,
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
                category,
                compressed_bytes: totals.compressed_bytes,
            }),
            highlight: None,
            child: None,
            details: false,
            details_expanded: false,
            exited: false,
        }
    }

    pub(crate) fn stands_for_one_process(&self) -> bool {
        self.section.is_none()
            && self.child.as_ref().is_none_or(Child::has_metrics)
            && !(self.has_children && self.is_expanded)
    }
}

pub(crate) fn pin_group(
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
mod tests {
    use super::*;

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
        collapsed_sections: &HashSet<ProcessCategory>,
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
            .filter_map(|d| d.section.as_ref().map(|s| s.category.id()))
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
    fn a_pinned_group_keeps_its_place_among_siblings_across_a_resort() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(2, "beta"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);
        let pinned = pin_group(&mut sections, Some(2), None);
        assert_eq!(pinned, Some((2, 1)), "nothing moves on the first sight");

        let mut resorted = one_section(vec![
            group(row(2, "beta"), vec![]),
            group(row(1, "alpha"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);
        let pinned = pin_group(&mut resorted, Some(2), pinned);

        assert_eq!(pinned, Some((2, 1)));
        assert_eq!(group_pids(&resorted), vec![1, 2, 3], "beta pulled back to its place");
    }

    #[test]
    fn a_new_selection_keeps_its_own_place() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(2, "beta"), vec![]),
            group(row(3, "gamma"), vec![]),
        ]);

        let pinned = pin_group(&mut sections, Some(3), Some((1, 0)));

        assert_eq!(pinned, Some((3, 2)));
        assert_eq!(group_pids(&sections), vec![1, 2, 3], "the clicked group does not jump");
    }

    #[test]
    fn a_pinned_group_never_lands_inside_an_expanded_group() {
        let mut sections = one_section(vec![
            group(row(10, "rust-analyzer.exe"), vec![row(11, "rust-analyzer.exe"), row(12, "rust-analyzer.exe")]),
            group(row(20, "proc-macro-srv.exe"), vec![]),
            group(row(30, "cargo.exe"), vec![]),
        ]);
        let pinned = pin_group(&mut sections, Some(20), None);

        let mut resorted = one_section(vec![
            group(row(30, "cargo.exe"), vec![]),
            group(row(10, "rust-analyzer.exe"), vec![row(11, "rust-analyzer.exe"), row(12, "rust-analyzer.exe")]),
            group(row(20, "proc-macro-srv.exe"), vec![]),
        ]);
        pin_group(&mut resorted, Some(20), pinned);
        let mut expanded = HashSet::new();
        expanded.insert("rust-analyzer.exe".to_string());
        let out = flat(&resorted, &expanded, &HashSet::new());

        assert_eq!(
            process_pids(&out),
            vec![30, 20, 10, 10, 11, 12],
            "the pinned group sits between whole groups, never among another group's members"
        );
    }

    #[test]
    fn selecting_a_member_pins_its_whole_group() {
        let mut sections = one_section(vec![
            group(row(1, "alpha"), vec![]),
            group(row(10, "chrome.exe"), vec![row(11, "chrome.exe")]),
        ]);

        assert_eq!(pin_group(&mut sections, Some(11), None), Some((11, 1)));
    }

    #[test]
    fn pin_reports_none_when_the_selection_is_not_on_screen() {
        let mut sections = one_section(vec![group(row(1, "alpha"), vec![])]);

        assert_eq!(pin_group(&mut sections, Some(99), Some((99, 0))), None);
        assert_eq!(pin_group(&mut sections, None, None), None);
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
        collapsed.insert(ProcessCategory::WindowsService);
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
                    Child::Window(title) => &**title,
                    Child::Console => &*d.row.display_name,
                };
                Some((d.depth, d.row.pid, label))
            })
            .collect()
    }

    #[test]
    fn an_app_lists_its_windows_before_its_services_even_a_single_one() {
        let mut app = hosting(7, "app.exe", &["Helper"]);
        app.windows = Some(Arc::from(vec![Arc::<str>::from("Main window")]));
        let sections = one_section(vec![group(app, vec![])]);

        let out = with_open(&sections, &[7]);

        assert_eq!(children(&out), vec![(2, 7, "Main window"), (2, 7, "Helper")]);
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
    fn a_service_row_is_never_the_selected_process() {
        let sections = one_section(vec![group(hosting(7, "svchost.exe", &["Audio"]), vec![])]);
        let mut out = with_open(&sections, &[7]);
        highlight(&mut out, Some(Selection::Process(7)));

        assert_eq!(highlighted(&out), vec![(1, 7, Highlight::Whole)]);
    }

    fn console_of(pid: u32, owner: u32) -> ProcessRow {
        let mut console = row(pid, "conhost.exe");
        console.display_name = "Console Window Host".into();
        console.owner_pid = Some(owner);
        console
    }

    fn sections_for(rows: &[ProcessRow]) -> Vec<Section> {
        let mut cache = GroupsCache::empty();
        cache.get(rows, &[], None, None, &cpu_order());
        cache.sections
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

    struct Screen {
        cache: GroupsCache,
        held: Held,
    }

    impl Screen {
        fn new() -> Self {
            Self {
                cache: GroupsCache::empty(),
                held: Held::default(),
            }
        }

        fn show(&mut self, rows: &[ProcessRow], selection: Option<Selection>) -> Vec<DisplayRow> {
            let pid = selection.map(|s| match s {
                Selection::Process(pid) | Selection::Group(pid) => pid,
            });
            let exited_rows = self.held.exited(selection, rows);
            let kept = self.held.section(selection);
            let exited: HashSet<u32> = exited_rows.iter().map(|r| r.pid).collect();
            let sections = self.cache.get(rows, &exited_rows, pid, kept, &cpu_order());
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

}
