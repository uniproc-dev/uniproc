use std::rc::Rc;

use app_contracts::features::agents::EnvironmentKind;
use app_contracts::features::processes::{
    HostedService, MachineSummary, ProcessCategory, ProcessColumn, ProcessRow, ProcessWindow,
};
use guicons::icon;
use guinea_widgets::table::ColumnSpec;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, ChildrenControl, Color, ContentControl, CornerRadius, EncodedImage, Grid,
    GridChildExt, GridLength, HorizontalAlignment, Image, LayoutControl, PointerEventInfo,
    ResourceOverrides, StackPanel, TextTrimming, TextWrapping, ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, opacity, radius, size, space, Palette};
use crate::widgets::distro_icon::distro_icon;
use crate::widgets::separator;
use crate::widgets::table_cell::{self, metric_cell, Heat, Highlight, Metric};
use crate::widgets::text::{body_strong, caption, text};

use guinea::winui::MarkExt;

use super::super::marks::ProcessesMark;
use super::column_layout::ColumnLayout;
use super::grouping::{is_service_host, Child, DisplayRow, ProcessName, SectionId, SectionRow, WslRow};

struct Hit;

#[expect(non_upper_case_globals)]
impl Hit {
    const Transparent: Color = Color::argb(0, 0, 0, 0);
}

struct Header;

#[expect(non_upper_case_globals)]
impl Header {
    const Padding: f64 = 8.0;
}

struct Cpu;

#[expect(non_upper_case_globals)]
impl Cpu {
    const Zero: f32 = 0.05;
    const HeatThreshold: f32 = 0.01;
}

pub(crate) fn sort_indicator_icon(descending: bool) -> View {
    if descending {
        icon!(chevron_down_regular).size(size::Chevron).build_element()
    } else {
        icon!(chevron_up_regular).size(size::Chevron).build_element()
    }
}

fn memory_heat_color(row: &ProcessRow, accent: Color, palette: Palette) -> Color {
    if &*row.name == ProcessName::MemoryCompression {
        palette.heat_muted
    } else {
        accent
    }
}

fn expand_chevron(expanded: bool) -> View {
    if expanded {
        icon!(chevron_down_regular).size(size::Chevron).build_element()
    } else {
        icon!(chevron_right_regular).size(size::Chevron).build_element()
    }
}

fn fallback_process_icon() -> View {
    icon!(app).size(size::Icon).build_element()
}

fn chevron_slot(content: View, on_press: Option<Callback<()>>, height: f64) -> View {
    let reach = NameLine::Spacing / 2.0;
    let slot = Border::new()
        .width(space::Cell + size::ChevronSlot + reach)
        .height(height)
        .padding(Thickness::new(space::Cell, 0.0, reach, 0.0))
        .margin(Thickness::new(-space::Cell, 0.0, -reach, 0.0))
        .vertical_alignment(VerticalAlignment::Center);
    match on_press {
        Some(on_press) => slot
            .mark(ProcessesMark::Chevron)
            .background(Hit::Transparent)
            .on_pointer_released(move |_: PointerEventInfo| {
                let _ = on_press.call(());
            })
            .content(content),
        None => slot.content(content),
    }
}

fn row_height(d: &DisplayRow) -> f64 {
    if d.section.is_some() {
        size::SectionRow
    } else {
        size::ProcessRow
    }
}

fn selection_bar() -> View {
    Border::new()
        .width(size::SelectionBarWidth)
        .height(size::SelectionBarHeight)
        .corner_radius(radius::SelectionBar)
        .background(ThemeBrush::Accent)
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::new(space::SelectionBar, 0.0, 0.0, 0.0))
        .into()
}

fn group_count(count: usize, palette: Palette) -> View {
    text(format!("({count})"))
        .foreground(palette.tertiary_text)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::new(space::Count, 0.0, 0.0, 0.0))
        .into()
}

struct NameLine {
    indent: f64,
    chevron: View,
    icon: Option<View>,
    label: View,
    count: View,
}

#[expect(non_upper_case_globals)]
impl NameLine {
    const Spacing: f64 = space::Control;
}

fn name_line(line: NameLine) -> View {
    let icon = match line.icon {
        Some(icon) => Border::new()
            .grid_column(1)
            .vertical_alignment(VerticalAlignment::Center)
            .margin(Thickness::new(NameLine::Spacing, 0.0, 0.0, 0.0))
            .content(icon)
            .into(),
        None => View::empty(),
    };
    let label_and_count = Grid::new()
        .grid_column(2)
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::new(NameLine::Spacing, 0.0, 0.0, 0.0))
        .children((
            Border::new().grid_column(0).content(line.label),
            Border::new().grid_column(1).content(line.count),
        ));
    Grid::new()
        .columns([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
        .margin(Thickness::new(space::Cell + line.indent, 0.0, 0.0, 0.0))
        .children((
            Border::new().grid_column(0).content(line.chevron),
            icon,
            label_and_count,
        ))
        .into()
}

#[derive(Clone)]
pub(crate) struct GroupByType {
    pub(crate) on: bool,
    pub(crate) toggle: Callback<()>,
}

#[derive(Clone)]
pub(crate) struct NameCellActions {
    pub(crate) icons: Rc<context::IconCache>,
    pub(crate) toggle_group: Callback<String>,
    pub(crate) toggle_process: Callback<u32>,
}

fn indent(depth: u8) -> f64 {
    f64::from(depth.saturating_sub(1)) * (size::Icon + NameLine::Spacing)
}

fn runs_from_package(exe_path: &str, package_full_name: &str) -> bool {
    let Some(name) = package_full_name.split('_').next().filter(|name| !name.is_empty()) else {
        return false;
    };
    let folder = format!("\\{}_", name.to_ascii_lowercase());
    exe_path.to_ascii_lowercase().contains(&folder)
}

fn process_icon(icons: &context::IconCache, row: &ProcessRow) -> View {
    if is_service_host(row) {
        return icon!(gears).size(size::Icon).build_element();
    }
    let package = Some(&*row.package_full_name)
        .filter(|package| runs_from_package(&row.exe_path, package));
    let png = icons.icon(context::IconRequest {
        path: &row.exe_path,
        package_full_name: package,
    });
    match png {
        Some(png) => png_icon(png),
        None => fallback_process_icon(),
    }
}

fn window_icon(icons: &context::IconCache, window: &ProcessWindow, owner: &ProcessRow) -> View {
    match icons.window_icon(window.handle) {
        Some(png) => png_icon(png),
        None => process_icon(icons, owner),
    }
}

fn png_icon(png: std::sync::Arc<[u8]>) -> View {
    Image::new()
        .width(size::Icon)
        .height(size::Icon)
        .vertical_alignment(VerticalAlignment::Center)
        .source_data(EncodedImage::new(png))
        .into()
}

struct NameCell<'a> {
    actions: &'a NameCellActions,
    l10n: &'a L10n,
    palette: Palette,
}

fn selection_bar_for(height: f64, band: Highlight) -> View {
    let reach = (height + size::SelectionBarHeight) / 2.0;
    let radius = radius::SelectionBar;
    let bar = Border::new()
        .width(size::SelectionBarWidth)
        .background(ThemeBrush::Accent)
        .horizontal_alignment(HorizontalAlignment::Left)
        .margin(Thickness::new(space::SelectionBar, 0.0, 0.0, 0.0))
        .corner_radius(CornerRadius::new(
            if band.top { radius } else { 0.0 },
            if band.top { radius } else { 0.0 },
            if band.bottom { radius } else { 0.0 },
            if band.bottom { radius } else { 0.0 },
        ));
    match (band.top, band.bottom) {
        (true, true) => return selection_bar(),
        (true, false) => bar.height(reach).vertical_alignment(VerticalAlignment::Bottom),
        (false, true) => bar.height(reach).vertical_alignment(VerticalAlignment::Top),
        (false, false) => bar.vertical_alignment(VerticalAlignment::Stretch),
    }
    .into()
}

fn header_frame() -> Grid {
    Grid::new()
        .height(size::TableHeader)
}

fn name_cell(cell: &NameCell<'_>, d: &DisplayRow) -> View {
    if let Some(section) = &d.section {
        return section_name_cell(cell, d, section);
    }
    match &d.child {
        Some(Child::Service(service)) => return service_name_cell(cell, d, service),
        Some(Child::Window(window)) => return window_name_cell(cell, d, window),
        Some(Child::Console) | None => {}
    }

    let chevron = if d.has_children {
        let toggle = cell.actions.toggle_group.clone();
        let group = match &d.wsl {
            Some(WslRow::Environment { key, .. }) => key.clone(),
            _ => d.row.name.to_string(),
        };
        chevron_slot(
            expand_chevron(d.is_expanded),
            Some(Callback::new(move |()| {
                let _ = toggle.call(group.clone());
            })),
            row_height(d),
        )
    } else if d.details {
        let toggle = cell.actions.toggle_process.clone();
        let pid = d.row.pid;
        chevron_slot(
            expand_chevron(d.details_expanded),
            Some(Callback::new(move |()| {
                let _ = toggle.call(pid);
            })),
            row_height(d),
        )
    } else {
        chevron_slot(View::empty(), None, row_height(d))
    };

    let note = |mark: ProcessesMark, label: String| -> View {
        caption(label)
            .mark(mark)
            .foreground(cell.palette.tertiary_text)
            .vertical_alignment(VerticalAlignment::Center)
            .margin(Thickness::new(space::Control, 0.0, 0.0, 0.0))
            .into()
    };
    let count = if d.absent {
        note(ProcessesMark::NotRunning, cell.l10n.processes_not_running())
    } else if d.exited {
        note(ProcessesMark::Exited, cell.l10n.processes_exited())
    } else if d.has_children {
        group_count(d.group_size, cell.palette)
    } else {
        View::empty()
    };
    let label = match &d.row.owner {
        Some(owner) if !d.has_children && d.child.is_none() => {
            format!("{owner} — {}", d.row.display_name)
        }
        _ if d.row.display_name.is_empty() && d.wsl.is_some() => cell.l10n.processes_wsl_other(),
        _ => d.row.display_name.to_string(),
    };
    let icon = match &d.wsl {
        Some(WslRow::Environment { kind: EnvironmentKind::DockerContainer, .. }) => {
            icon!(docker).size(size::Icon).build_element()
        }
        Some(WslRow::Environment { .. }) => distro_icon(&d.row.name),
        Some(WslRow::Process { .. }) => icon!(proc_regular).size(size::Icon).build_element(),
        None => process_icon(&cell.actions.icons, &d.row),
    };

    let line = name_line(NameLine {
        indent: indent(d.depth),
        chevron,
        icon: Some(icon),
        label: table_cell::cell_text(label)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count,
    });

    let line: View = if d.exited || d.absent {
        Border::new().opacity(opacity::Stopped).content(line).into()
    } else {
        line
    };

    name_row(d, line, cell.palette)
}

fn rule_below(d: &DisplayRow, palette: Palette) -> View {
    if d.rule_below {
        separator(palette)
            .mark(ProcessesMark::PinnedRule)
            .vertical_alignment(VerticalAlignment::Bottom)
            .into()
    } else {
        View::empty()
    }
}

fn name_row(d: &DisplayRow, line: View, palette: Palette) -> View {
    let bar = match d.highlight {
        Some(band) => selection_bar_for(row_height(d), band),
        None => View::empty(),
    };
    Grid::new()
        .height(row_height(d))
        .children((bar, line, rule_below(d, palette)))
        .into()
}

fn service_name_cell(cell: &NameCell<'_>, d: &DisplayRow, service: &HostedService) -> View {
    let line = name_line(NameLine {
        indent: indent(d.depth),
        chevron: chevron_slot(View::empty(), None, row_height(d)),
        icon: Some(icon!(gears).size(size::Icon).build_element()),
        label: table_cell::cell_text(&*service.display_name)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count: View::empty(),
    });

    name_row(d, line, cell.palette)
}

fn window_name_cell(cell: &NameCell<'_>, d: &DisplayRow, window: &ProcessWindow) -> View {
    let title = if window.title.is_empty() {
        d.row.display_name.to_string()
    } else {
        window.title.to_string()
    };
    let line = name_line(NameLine {
        indent: indent(d.depth),
        chevron: chevron_slot(View::empty(), None, row_height(d)),
        icon: Some(window_icon(&cell.actions.icons, window, &d.row)),
        label: table_cell::cell_text(title)
            .foreground(cell.palette.secondary_text)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count: View::empty(),
    });

    name_row(d, line, cell.palette)
}

fn section_name_cell(cell: &NameCell<'_>, d: &DisplayRow, section: &SectionRow) -> View {
    let line = name_line(NameLine {
        indent: 0.0,
        chevron: chevron_slot(expand_chevron(d.is_expanded), None, row_height(d)),
        icon: None,
        label: body_strong(section_label(cell.l10n, section.id))
            .text_wrapping(TextWrapping::NoWrap)
            .text_trimming(TextTrimming::CharacterEllipsis)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count: group_count(d.group_size, cell.palette),
    });

    Grid::new()
        .height(row_height(d))
        .children((line,))
        .into()
}

fn sort_mark(sorted: Option<bool>) -> View {
    match sorted {
        Some(descending) => Border::new()
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Top)
            .margin(Thickness::new(0.0, -Header::Padding, 0.0, 0.0))
            .content(sort_indicator_icon(descending)),
        None => View::empty(),
    }
}

fn group_by_type_toggle(group_by_type: &GroupByType) -> View {
    let toggle = group_by_type.toggle.clone();
    let icon = if group_by_type.on {
        icon!(group_list_filled)
    } else {
        icon!(group_list_regular)
    };
    Button::new()
        .mark(ProcessesMark::GroupByType)
        .style(ButtonStyle::Subtle)
        .resource_overrides(ResourceOverrides::new().set("ButtonPadding", Thickness::uniform(space::Compact)))
        .horizontal_alignment(HorizontalAlignment::Right)
        .vertical_alignment(VerticalAlignment::Bottom)
        .margin(Thickness::new(0.0, 0.0, space::Cell, 0.0))
        .on_click(move || {
            let _ = toggle.call(());
        })
        .content(icon.size(size::Icon).build_element())
        .into()
}

fn name_header(label: String, sorted: Option<bool>, group_by_type: &GroupByType, palette: Palette) -> View {
    header_frame().children((
        sort_mark(sorted),
        caption(label)
            .foreground(palette.tertiary_text)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::new(
                space::Cell + size::ChevronSlot + NameLine::Spacing,
                0.0,
                0.0,
                0.0,
            )),
        group_by_type_toggle(group_by_type),
    ))
}

fn metric_header(
    label: String,
    value: String,
    sorted: Option<bool>,
    palette: Palette,
) -> View {
    header_frame().children((
        sort_mark(sorted),
        StackPanel::new()
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::xy(space::Cell, 0.0))
            .children((
                text(value)
                    .text_wrapping(TextWrapping::NoWrap)
                    .text_trimming(TextTrimming::CharacterEllipsis)
                    .horizontal_alignment(HorizontalAlignment::Right),
                caption(label)
                    .foreground(palette.tertiary_text)
                    .text_wrapping(TextWrapping::NoWrap)
                    .text_trimming(TextTrimming::CharacterEllipsis)
                    .horizontal_alignment(HorizontalAlignment::Right),
            )),
    ))
}

#[derive(Clone, Copy)]
struct Place {
    width: f64,
    min_width: f64,
    sorted: Option<bool>,
    palette: Palette,
}

type Column = ColumnSpec<DisplayRow, ProcessColumn>;

fn name_column(place: Place, actions: NameCellActions, group_by_type: GroupByType, l10n: L10n) -> Column {
    let header_l10n = l10n.clone();
    let (sorted, palette) = (place.sorted, place.palette);
    let header = move || {
        name_header(header_l10n.processes_col_name(), sorted, &group_by_type, palette)
    };
    ColumnSpec::new_with_header(ProcessColumn::Name, header, place.width, move |d: &DisplayRow| {
        let cell = NameCell {
            actions: &actions,
            l10n: &l10n,
            palette,
        };
        name_cell(&cell, d)
    })
    .min_width(place.min_width)
    .flush()
    .sortable()
}

struct MetricColumn<F, H> {
    id: ProcessColumn,
    label: String,
    total: String,
    place: Place,
    value: F,
    heat: H,
    threshold: f32,
}

fn metric_column<F, H>(column: MetricColumn<F, H>) -> Column
where
    F: Fn(&ProcessRow) -> (String, bool) + 'static,
    H: Fn(&ProcessRow) -> (f32, Color) + 'static,
{
    let MetricColumn {
        id,
        label,
        total,
        place,
        value,
        heat,
        threshold,
    } = column;
    let (sorted, palette) = (place.sorted, place.palette);
    let header = move || metric_header(label.clone(), total.clone(), sorted, palette);
    ColumnSpec::new_with_header(id, header, place.width, move |d: &DisplayRow| {
        if d.absent || d.child.as_ref().is_some_and(|child| !child.has_metrics()) {
            return Grid::new()
                .height(row_height(d))
                .children((rule_below(d, palette),))
                .into();
        }
        let (text, zero) = value(&d.row);
        let heat = if d.section.is_some() || zero {
            None
        } else {
            let (share, color) = heat(&d.row);
            Some(Heat {
                share,
                threshold,
                color,
            })
        };
        let cell = metric_cell(
            Metric {
                text,
                zero,
                heat,
                height: row_height(d),
            },
            palette,
        );
        if d.rule_below {
            Grid::new().children((cell, rule_below(d, palette))).into()
        } else {
            cell
        }
    })
    .min_width(place.min_width)
    .flush()
    .sortable()
}

fn percent(value: f32) -> String {
    format!("{value:.1}%")
}

pub(crate) struct ColumnInputs<'a> {
    pub(crate) layout: &'a ColumnLayout,
    pub(crate) machine: Option<MachineSummary>,
    pub(crate) rows: &'a [ProcessRow],
    pub(crate) actions: NameCellActions,
    pub(crate) group_by_type: GroupByType,
    pub(crate) sort_column: ProcessColumn,
    pub(crate) descending: bool,
    pub(crate) palette: Palette,
    pub(crate) l10n: &'a L10n,
}

pub(crate) fn build_columns(inputs: ColumnInputs<'_>) -> Vec<Column> {
    let ColumnInputs {
        layout,
        machine,
        rows,
        actions,
        group_by_type,
        sort_column,
        descending,
        palette,
        l10n,
    } = inputs;

    let net_max = rows.iter().map(|r| r.net_bytes).max().unwrap_or(0).max(1) as f32;
    let disk_max = rows.iter().map(|r| r.disk_bytes).max().unwrap_or(0).max(1) as f32;
    let net_total: u64 = rows.iter().map(|r| r.net_bytes).sum();
    let disk_total: u64 = rows.iter().map(|r| r.disk_bytes).sum();
    let memory_total_bytes = machine.as_ref().map_or(0, |m| m.memory_total_bytes);
    let cpu_total = machine.as_ref().map(|m| percent(m.cpu_percent)).unwrap_or_default();
    let memory_used = machine
        .as_ref()
        .filter(|m| m.memory_total_bytes > 0)
        .map(|m| percent(m.memory_used_bytes as f32 / m.memory_total_bytes as f32 * 100.0))
        .unwrap_or_default();
    let accent = accent_color();
    let visible: Vec<_> = layout
        .columns()
        .into_iter()
        .filter(|column| column.visible)
        .collect();

    visible
        .into_iter()
        .map(|column| {
            let place = Place {
                width: column.width,
                min_width: column.min_width,
                sorted: (sort_column == column.column).then_some(descending),
                palette,
            };
            match column.column {
                ProcessColumn::Name => {
                    name_column(place, actions.clone(), group_by_type.clone(), l10n.clone())
                }
                ProcessColumn::Cpu => metric_column(MetricColumn {
                    id: ProcessColumn::Cpu,
                    label: l10n.processes_col_cpu(),
                    total: cpu_total.clone(),
                    place,
                    value: |r: &ProcessRow| (percent(r.cpu_percent), r.cpu_percent < Cpu::Zero),
                    heat: move |r: &ProcessRow| (r.cpu_percent / 100.0, accent),
                    threshold: Cpu::HeatThreshold,
                }),
                ProcessColumn::Memory => metric_column(MetricColumn {
                    id: ProcessColumn::Memory,
                    label: l10n.processes_col_memory(),
                    total: memory_used.clone(),
                    place,
                    value: |r: &ProcessRow| (format::bytes(r.memory_bytes), r.memory_bytes == 0),
                    heat: move |r: &ProcessRow| {
                        let share = if memory_total_bytes > 0 {
                            r.memory_bytes as f32 / memory_total_bytes as f32
                        } else {
                            0.0
                        };
                        (share, memory_heat_color(r, accent, palette))
                    },
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Net => metric_column(MetricColumn {
                    id: ProcessColumn::Net,
                    label: l10n.processes_col_net(),
                    total: format::bytes_per_second(net_total),
                    place,
                    value: |r: &ProcessRow| (format::bytes_per_second(r.net_bytes), r.net_bytes == 0),
                    heat: move |r: &ProcessRow| (r.net_bytes as f32 / net_max, accent),
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Disk => metric_column(MetricColumn {
                    id: ProcessColumn::Disk,
                    label: l10n.processes_col_disk(),
                    total: format::bytes_per_second(disk_total),
                    place,
                    value: |r: &ProcessRow| (format::bytes_per_second(r.disk_bytes), r.disk_bytes == 0),
                    heat: move |r: &ProcessRow| (r.disk_bytes as f32 / disk_max, accent),
                    threshold: Heat::Threshold,
                }),
            }
        })
        .collect()
}

pub(crate) fn section_label(l10n: &L10n, id: SectionId) -> String {
    match id {
        SectionId::Pinned => l10n.processes_category_pinned(),
        SectionId::Category(category) => category_label(l10n, category),
    }
}

pub(crate) fn category_label(l10n: &L10n, category: ProcessCategory) -> String {
    match category {
        ProcessCategory::App => l10n.processes_category_app(),
        ProcessCategory::BackgroundThirdParty => l10n.processes_category_background_third_party(),
        ProcessCategory::Wsl => l10n.processes_category_wsl(),
        ProcessCategory::BackgroundMicrosoft => l10n.processes_category_background_microsoft(),
        ProcessCategory::WindowsService => l10n.processes_category_windows_service(),
        ProcessCategory::WindowsKernel => l10n.processes_category_windows_kernel(),
    }
}

#[cfg(test)]
mod tests {
    use super::runs_from_package;

    #[test]
    fn a_packaged_app_takes_its_package_icon() {
        assert!(runs_from_package(
            r"C:\Program Files\WindowsApps\Claude_2.9939.2.0_x64__pzs8sxrjxfjjc\app\claude.exe",
            "Claude_2.9939.2.0_x64__pzs8sxrjxfjjc",
        ));
        assert!(runs_from_package(
            r"C:\Windows\SystemApps\MicrosoftWindows.Client.CBS_cw5n1h2txyewy\SearchHost.exe",
            "MicrosoftWindows.Client.CBS_1000.26100.360.0_x64__cw5n1h2txyewy",
        ));
    }

    #[test]
    fn a_broker_running_for_a_package_keeps_its_own_icon() {
        assert!(!runs_from_package(
            r"C:\Windows\System32\RuntimeBroker.exe",
            "Claude_2.9939.2.0_x64__pzs8sxrjxfjjc",
        ));
        assert!(!runs_from_package(r"C:\Windows\System32\RuntimeBroker.exe", ""));
    }
}
