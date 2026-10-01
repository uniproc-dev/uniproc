use std::rc::Rc;

use app_contracts::features::agents::{Architecture, EnvironmentKind, Isolation};
use app_contracts::features::processes::{
    HostedService, MachineSummary, ProcessCategory, ProcessColumn, ProcessRow, ProcessStatus, ProcessWindow,
};
use app_contracts::features::settings::Units;
use guicons::icon;
use crate::widgets::table::ColumnSpec;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, ChildrenControl, Color, ContentControl, CornerRadius, EncodedImage, Grid,
    GridChildExt, GridLength, HorizontalAlignment, Image, LayoutControl, PointerEventInfo,
    ResourceOverrides, StackPanel, TextTrimming, TextWrapping, ThemeBrush, Thickness, Tooltip, TooltipExt,
    VerticalAlignment, View,
};

use crate::format::{self, percent, Rate};
use crate::l10n::L10n;
use crate::theme::{accent_color, size, space, Palette};
use crate::widgets::distro_icon::distro_icon;
use crate::widgets::separator;
use crate::widgets::table_cell::{self, metric_cell, Heat, Highlight, Metric};
use crate::widgets::text::{body_strong, caption, text};

use guinea::winui::MarkExt;

use super::super::marks::ProcessesMark;
use super::column_layout::ColumnLayout;
use super::grouping::{is_idle, is_service_host, Child, DisplayRow, DropEdge, ProcessName, SectionId, SectionRow, WslRow};
use super::section_drag::SectionGesture;

struct Header;

#[expect(non_upper_case_globals)]
impl Header {
    const Padding: f64 = 8.0;
    const Height: f64 = 36.0;
}

struct Chevron;

#[expect(non_upper_case_globals)]
impl Chevron {
    const Size: f64 = 10.0;
    const Slot: f64 = 14.0;
}

struct SelectionBar;

#[expect(non_upper_case_globals)]
impl SelectionBar {
    const Width: f64 = 3.0;
    const Height: f64 = 16.0;
    const Radius: f64 = 2.0;
    const Inset: f64 = 5.0;
}

struct DropLine;

#[expect(non_upper_case_globals)]
impl DropLine {
    const Thickness: f64 = 2.0;
}

struct Cpu;

#[expect(non_upper_case_globals)]
impl Cpu {
    const Zero: f32 = 0.05;
    const HeatThreshold: f32 = 0.01;
}

pub(crate) fn sort_indicator_icon(descending: bool) -> View {
    if descending {
        icon!(chevron_down_regular).size(Chevron::Size).build_element()
    } else {
        icon!(chevron_up_regular).size(Chevron::Size).build_element()
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
        icon!(chevron_down_regular).size(Chevron::Size).build_element()
    } else {
        icon!(chevron_right_regular).size(Chevron::Size).build_element()
    }
}

fn fallback_process_icon() -> View {
    icon!(app).size(size::Icon).build_element()
}

fn chevron_slot(content: View, on_press: Option<Callback<()>>, height: f64) -> View {
    let reach = NameLine::Spacing / 2.0;
    let slot = Border::new()
        .width(space::Cell + Chevron::Slot + reach)
        .height(height)
        .padding(Thickness::new(space::Cell, 0.0, reach, 0.0))
        .margin(Thickness::new(-space::Cell, 0.0, -reach, 0.0))
        .vertical_alignment(VerticalAlignment::Center);
    match on_press {
        Some(on_press) => slot
            .mark(ProcessesMark::Chevron)
            .background(Color::transparent())
            .on_pointer_released(move |_: PointerEventInfo| {
                let _ = on_press.call(());
            })
            .content(content),
        None => slot.content(content),
    }
}

fn selection_bar() -> View {
    Border::new()
        .width(SelectionBar::Width)
        .height(SelectionBar::Height)
        .corner_radius(SelectionBar::Radius)
        .background(ThemeBrush::Accent)
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::new(SelectionBar::Inset, 0.0, 0.0, 0.0))
        .into()
}

fn group_count(count: usize, l10n: &L10n, palette: Palette) -> View {
    text(l10n.processes_group_count(count as i64))
        .foreground(palette.tertiary_text)
        .vertical_alignment(VerticalAlignment::Center)
        .margin(Thickness::new(space::Compact, 0.0, 0.0, 0.0))
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
    pub(crate) section_gesture: Callback<SectionGesture>,
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
        return table_cell::service_icon();
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
    let reach = (height + SelectionBar::Height) / 2.0;
    let radius = SelectionBar::Radius;
    let bar = Border::new()
        .width(SelectionBar::Width)
        .background(ThemeBrush::Accent)
        .horizontal_alignment(HorizontalAlignment::Left)
        .margin(Thickness::new(SelectionBar::Inset, 0.0, 0.0, 0.0))
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
        .height(Header::Height)
}

fn with_column_menu(menu: &Callback<()>, header: impl Into<View>) -> View {
    let menu = menu.clone();
    Border::new()
        .background(Color::transparent())
        .on_pointer_pressed(Callback::new(move |pointer: PointerEventInfo| {
            if pointer.is_right_button_pressed {
                let _ = menu.call(());
            }
        }))
        .content(header.into())
        .into()
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

    let chevron = if matches!(d.wsl, Some(WslRow::Environment { .. })) {
        chevron_slot(expand_chevron(d.is_expanded), None, d.height())
    } else if d.has_children {
        let toggle = cell.actions.toggle_group.clone();
        let group = d.row.name.to_string();
        chevron_slot(
            expand_chevron(d.is_expanded),
            Some(Callback::new(move |()| {
                let _ = toggle.call(group.clone());
            })),
            d.height(),
        )
    } else if d.details {
        let toggle = cell.actions.toggle_process.clone();
        let pid = d.row.pid;
        chevron_slot(
            expand_chevron(d.details_expanded),
            Some(Callback::new(move |()| {
                let _ = toggle.call(pid);
            })),
            d.height(),
        )
    } else {
        chevron_slot(View::empty(), None, d.height())
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
        group_count(d.group_size, cell.l10n, cell.palette)
    } else {
        View::empty()
    };
    let label = match (&d.row.owner, &d.wsl) {
        (Some(owner), _) if !d.has_children && d.child.is_none() => {
            cell.l10n.processes_owned_name(
                owner.others as i64,
                owner.name.to_string(),
                d.row.display_name.to_string(),
            )
        }
        (_, Some(WslRow::Environment { pid_ns, .. })) if d.row.display_name.is_empty() => {
            cell.l10n.processes_wsl_namespace(pid_ns.to_string())
        }
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

    let depth = match d.wsl {
        Some(WslRow::Process { .. }) => d.depth.saturating_sub(1),
        _ => d.depth,
    };
    let line = name_line(NameLine {
        indent: indent(depth),
        chevron,
        icon: Some(icon),
        label: table_cell::cell_text(label)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count,
    });

    let line = table_cell::dimmed(line, d.exited || d.absent);

    name_row(d, line, cell.palette)
}

fn rules(d: &DisplayRow, palette: Palette) -> View {
    if !has_rules(d) {
        return View::empty();
    }
    let pinned: View = if d.rule_below {
        separator(palette)
            .mark(ProcessesMark::PinnedRule)
            .vertical_alignment(VerticalAlignment::Bottom)
            .into()
    } else {
        View::empty()
    };
    let drop: View = match d.drop_edge {
        Some(edge) => Border::new()
            .mark(ProcessesMark::DropLine)
            .height(DropLine::Thickness)
            .background(ThemeBrush::Accent)
            .vertical_alignment(match edge {
                DropEdge::Above => VerticalAlignment::Top,
                DropEdge::Below => VerticalAlignment::Bottom,
            })
            .into(),
        None => View::empty(),
    };
    Grid::new().children((pinned, drop)).into()
}

fn has_rules(d: &DisplayRow) -> bool {
    d.rule_below || d.drop_edge.is_some()
}

fn name_row(d: &DisplayRow, line: View, palette: Palette) -> View {
    let bar = match d.highlight {
        Some(band) => selection_bar_for(d.height(), band),
        None => View::empty(),
    };
    Grid::new()
        .height(d.height())
        .children((bar, line, rules(d, palette)))
        .into()
}

fn service_name_cell(cell: &NameCell<'_>, d: &DisplayRow, service: &HostedService) -> View {
    let line = name_line(NameLine {
        indent: indent(d.depth),
        chevron: chevron_slot(View::empty(), None, d.height()),
        icon: Some(table_cell::service_icon()),
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
        chevron: chevron_slot(View::empty(), None, d.height()),
        icon: Some(window_icon(&cell.actions.icons, window, &d.row)),
        label: table_cell::cell_text(title)
            .foreground(cell.palette.secondary_text)
            .vertical_alignment(VerticalAlignment::Center)
            .into(),
        count: View::empty(),
    });

    name_row(d, line, cell.palette)
}

fn wsl_notes(l10n: &L10n) -> Tooltip {
    let note = |line: String| -> View { text(line).text_wrapping(TextWrapping::Wrap).into() };
    Tooltip::rich(StackPanel::new().spacing(space::Compact).children((
        note(l10n.processes_wsl_note_heading()),
        note(l10n.processes_wsl_note_rows()),
        note(l10n.processes_wsl_note_memory()),
    )))
}

fn section_name_cell(cell: &NameCell<'_>, d: &DisplayRow, section: &SectionRow) -> View {
    let label = body_strong(section_label(cell.l10n, section.id))
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .vertical_alignment(VerticalAlignment::Center);
    let label: View = if section.id == SectionId::Category(ProcessCategory::Wsl) {
        label
            .mark(ProcessesMark::WslNotes)
            .tooltip_with(wsl_notes(cell.l10n))
    } else {
        label.into()
    };
    let line = name_line(NameLine {
        indent: 0.0,
        chevron: chevron_slot(expand_chevron(d.is_expanded), None, d.height()),
        icon: None,
        label,
        count: group_count(d.group_size, cell.l10n, cell.palette),
    });
    let line = table_cell::dimmed(line, d.lifted);

    section_grip(&cell.actions.section_gesture, section.id)
        .content(
            Grid::new()
                .height(d.height())
                .children((line, rules(d, cell.palette))),
        )
        .into()
}

fn section_grip(gesture: &Callback<SectionGesture>, section: SectionId) -> Border {
    let (pressed, moved, released, lost) = (gesture.clone(), gesture.clone(), gesture.clone(), gesture.clone());
    Border::new()
        .mark(ProcessesMark::SectionGrip)
        .background(Color::transparent())
        .capture_pointer_on_press(true)
        .on_pointer_pressed(move |pointer: PointerEventInfo| {
            if pointer.is_left_button_pressed {
                let _ = pressed.call(SectionGesture::Grab {
                    section,
                    at: pointer.window_y,
                    offset: pointer.y,
                });
            }
        })
        .on_pointer_moved(move |pointer: PointerEventInfo| {
            if pointer.is_left_button_pressed {
                let _ = moved.call(SectionGesture::Move { at: pointer.window_y });
            }
        })
        .on_pointer_released(move |_: PointerEventInfo| {
            let _ = released.call(SectionGesture::Release);
        })
        .on_pointer_capture_lost(move || {
            let _ = lost.call(SectionGesture::Lost);
        })
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

fn name_header(label: String, place: &Place, group_by_type: &GroupByType) -> View {
    let (sorted, palette) = (place.sorted, place.palette);
    with_column_menu(&place.menu, header_frame().children((
        sort_mark(sorted),
        caption(label)
            .foreground(palette.tertiary_text)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::new(
                space::Cell + Chevron::Slot + NameLine::Spacing,
                0.0,
                0.0,
                0.0,
            )),
        group_by_type_toggle(group_by_type),
    )))
}

fn metric_header(label: String, value: String, place: &Place) -> View {
    let (sorted, palette) = (place.sorted, place.palette);
    with_column_menu(&place.menu, header_frame().children((
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
    )))
}

fn text_header(label: String, place: &Place, align: HorizontalAlignment) -> View {
    with_column_menu(&place.menu, header_frame().children((
        sort_mark(place.sorted),
        caption(label)
            .foreground(place.palette.tertiary_text)
            .text_wrapping(TextWrapping::NoWrap)
            .text_trimming(TextTrimming::CharacterEllipsis)
            .horizontal_alignment(align)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::xy(space::Cell, 0.0)),
    )))
}

#[derive(Clone)]
struct Place {
    width: f64,
    min_width: f64,
    sorted: Option<bool>,
    palette: Palette,
    menu: Callback<()>,
}

type Column = ColumnSpec<DisplayRow, ProcessColumn>;

fn pid_value(d: &DisplayRow) -> Option<String> {
    if d.section.is_some() || d.absent {
        return None;
    }
    match (&d.wsl, &d.child) {
        (Some(WslRow::Environment { .. }), _) => None,
        (Some(WslRow::Process { .. }), _) => Some(d.row.pid.to_string()),
        (None, Some(Child::Window(_) | Child::Service(_))) => None,
        (None, _) if d.has_children => None,
        (None, _) => Some(d.row.pid.to_string()),
    }
}

fn process_name_value(d: &DisplayRow) -> Option<String> {
    if d.section.is_some() {
        return None;
    }
    match (&d.wsl, &d.child) {
        (Some(WslRow::Environment { .. }), _) => None,
        (None, Some(Child::Window(_) | Child::Service(_))) => None,
        _ => Some(d.row.name.to_string()),
    }
}

fn text_column<F>(id: ProcessColumn, label: String, place: Place, align: HorizontalAlignment, value: F) -> Column
where
    F: Fn(&DisplayRow) -> Option<String> + 'static,
{
    let (width, min_width, palette) = (place.width, place.min_width, place.palette);
    let header = move || text_header(label.clone(), &place, align);
    ColumnSpec::new_with_header(id, header, width, move |d: &DisplayRow| {
        let text: View = match value(d) {
            Some(value) => table_cell::cell_text(value)
                .horizontal_alignment(align)
                .vertical_alignment(VerticalAlignment::Center)
                .margin(Thickness::xy(space::Cell, 0.0))
                .into(),
            None => View::empty(),
        };
        Grid::new()
            .height(d.height())
            .children((text, rules(d, palette)))
            .into()
    })
    .min_width(min_width)
    .flush()
    .sortable()
}

pub(crate) fn column_label(l10n: &L10n, column: ProcessColumn) -> String {
    match column {
        ProcessColumn::Name => l10n.processes_col_name(),
        ProcessColumn::Pid => l10n.processes_col_pid(),
        ProcessColumn::ProcessName => l10n.processes_col_process_name(),
        ProcessColumn::Cpu => l10n.processes_col_cpu(),
        ProcessColumn::Memory => l10n.processes_col_memory(),
        ProcessColumn::Net => l10n.processes_col_net(),
        ProcessColumn::Disk => l10n.processes_col_disk(),
        ProcessColumn::Gpu => l10n.processes_col_gpu(),
        ProcessColumn::GpuMemory => l10n.processes_col_gpu_memory(),
        ProcessColumn::Status => l10n.processes_col_status(),
        ProcessColumn::Publisher => l10n.processes_col_publisher(),
        ProcessColumn::User => l10n.processes_col_user(),
        ProcessColumn::CommandLine => l10n.processes_col_command_line(),
        ProcessColumn::ImagePath => l10n.processes_col_image_path(),
        ProcessColumn::GpuEngine => l10n.processes_col_gpu_engine(),
        ProcessColumn::Platform => l10n.processes_col_platform(),
        ProcessColumn::Elevated => l10n.processes_col_elevated(),
        ProcessColumn::Isolation => l10n.processes_col_isolation(),
    }
}

#[derive(Clone, Copy)]
enum DetailScope {
    Process,
    Group,
}

fn detail_row(d: &DisplayRow, scope: DetailScope) -> Option<&ProcessRow> {
    if d.section.is_some() || d.absent {
        return None;
    }
    match (&d.wsl, &d.child, scope) {
        (Some(WslRow::Environment { .. }), _, _) => None,
        (None, Some(Child::Window(_) | Child::Service(_)), _) => None,
        (None, _, DetailScope::Process) if d.has_children => None,
        _ => Some(&d.row),
    }
}

fn detail_column<F>(id: ProcessColumn, l10n: &L10n, place: Place, scope: DetailScope, text: F) -> Column
where
    F: Fn(&L10n, &ProcessRow) -> String + 'static,
{
    let words = l10n.clone();
    text_column(id, column_label(l10n, id), place, HorizontalAlignment::Left, move |d| {
        detail_row(d, scope).map(|row| text(&words, row)).filter(|shown| !shown.is_empty())
    })
}

fn status_label(l10n: &L10n, status: ProcessStatus) -> String {
    match status {
        ProcessStatus::Running => String::new(),
        ProcessStatus::Suspended => l10n.processes_status_suspended(),
        ProcessStatus::Efficiency => l10n.processes_status_efficiency(),
    }
}

fn platform_label(l10n: &L10n, architecture: Architecture) -> String {
    match architecture {
        Architecture::Unknown => String::new(),
        Architecture::X86 => l10n.processes_platform_x86(),
        Architecture::X64 => l10n.processes_platform_x64(),
        Architecture::Arm => l10n.processes_platform_arm(),
        Architecture::Arm64 => l10n.processes_platform_arm64(),
        Architecture::Arm64X86Compatible => l10n.processes_platform_arm64_x86(),
        Architecture::Arm64X64Compatible => l10n.processes_platform_arm64_x64(),
    }
}

fn elevated_label(l10n: &L10n, elevated: Option<bool>) -> String {
    match elevated {
        Some(true) => l10n.processes_elevated_yes(),
        Some(false) => l10n.processes_elevated_no(),
        None => String::new(),
    }
}

fn isolation_label(l10n: &L10n, isolation: Isolation) -> String {
    match isolation {
        Isolation::Unknown | Isolation::None => String::new(),
        Isolation::AppContainer => l10n.processes_isolation_app_container(),
        Isolation::Uwp => l10n.processes_isolation_uwp(),
        Isolation::Silo => l10n.processes_isolation_silo(),
    }
}

fn gpu_engine_label(l10n: &L10n, row: &ProcessRow) -> String {
    row.details
        .gpu_engine
        .as_ref()
        .map(|label| l10n.processes_gpu_engine(label.adapter.to_string(), label.engine.to_string()))
        .unwrap_or_default()
}

fn name_column(place: Place, actions: NameCellActions, group_by_type: GroupByType, l10n: L10n) -> Column {
    let header_l10n = l10n.clone();
    let (width, min_width, palette) = (place.width, place.min_width, place.palette);
    let header = move || name_header(header_l10n.processes_col_name(), &place, &group_by_type);
    ColumnSpec::new_with_header(ProcessColumn::Name, header, width, move |d: &DisplayRow| {
        let cell = NameCell {
            actions: &actions,
            l10n: &l10n,
            palette,
        };
        name_cell(&cell, d)
    })
    .min_width(min_width)
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
    let (width, min_width, palette) = (place.width, place.min_width, place.palette);
    let header = move || metric_header(label.clone(), total.clone(), &place);
    ColumnSpec::new_with_header(id, header, width, move |d: &DisplayRow| {
        if d.absent || d.child.as_ref().is_some_and(|child| !child.has_metrics()) {
            return Grid::new()
                .height(d.height())
                .children((rules(d, palette),))
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
                height: d.height(),
            },
            palette,
        );
        if has_rules(d) {
            Grid::new().children((cell, rules(d, palette))).into()
        } else {
            cell
        }
    })
    .min_width(min_width)
    .flush()
    .sortable()
}

pub(crate) struct ColumnInputs<'a> {
    pub(crate) layout: &'a ColumnLayout,
    pub(crate) machine: Option<MachineSummary>,
    pub(crate) rows: &'a [ProcessRow],
    pub(crate) memory_as_percent: bool,
    pub(crate) units: Units,
    pub(crate) actions: NameCellActions,
    pub(crate) group_by_type: GroupByType,
    pub(crate) sort_column: ProcessColumn,
    pub(crate) descending: bool,
    pub(crate) palette: Palette,
    pub(crate) l10n: &'a L10n,
    pub(crate) header_menu: Callback<()>,
}

pub(crate) fn build_columns(inputs: ColumnInputs<'_>) -> Vec<Column> {
    let ColumnInputs {
        layout,
        machine,
        rows,
        memory_as_percent,
        units,
        actions,
        group_by_type,
        sort_column,
        descending,
        palette,
        l10n,
        header_menu,
    } = inputs;
    let network = Rate::network(units);
    let disk = Rate::disk(units);
    let units = units.bytes;

    let net_max =rows.iter().map(|r| r.net_bytes).max().unwrap_or(0).max(1) as f32;
    let disk_max = rows.iter().map(|r| r.disk_bytes).max().unwrap_or(0).max(1) as f32;
    let net_total: u64 = rows.iter().map(|r| r.net_bytes).sum();
    let disk_total: u64 = rows.iter().map(|r| r.disk_bytes).sum();
    let memory_total_bytes = machine.as_ref().map_or(0, |m| m.memory_total_bytes);
    let cpu_total = machine.as_ref().map(|m| percent(m.cpu_percent)).unwrap_or_default();
    let gpu_total = machine.as_ref().map(|m| percent(m.gpu_percent)).unwrap_or_default();
    let gpu_memory_total = machine
        .as_ref()
        .map(|m| format::bytes(units, m.gpu_memory_used_bytes))
        .unwrap_or_default();
    let gpu_memory_max = rows.iter().map(|r| r.gpu_memory_bytes).max().unwrap_or(0).max(1) as f32;
    let memory_share = move |bytes: u64| {
        if memory_total_bytes > 0 {
            bytes as f32 / memory_total_bytes as f32
        } else {
            0.0
        }
    };
    let memory_used = match (machine.as_ref(), memory_as_percent) {
        (Some(m), true) if m.memory_total_bytes > 0 => percent(memory_share(m.memory_used_bytes) * 100.0),
        (Some(m), false) => format::bytes(units, m.memory_used_bytes),
        _ => String::new(),
    };
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
                menu: header_menu.clone(),
            };
            match column.column {
                ProcessColumn::Name => {
                    name_column(place, actions.clone(), group_by_type.clone(), l10n.clone())
                }
                ProcessColumn::Pid => text_column(
                    ProcessColumn::Pid,
                    l10n.processes_col_pid(),
                    place,
                    HorizontalAlignment::Right,
                    pid_value,
                ),
                ProcessColumn::ProcessName => text_column(
                    ProcessColumn::ProcessName,
                    l10n.processes_col_process_name(),
                    place,
                    HorizontalAlignment::Left,
                    process_name_value,
                ),
                ProcessColumn::Cpu => metric_column(MetricColumn {
                    id: ProcessColumn::Cpu,
                    label: l10n.processes_col_cpu(),
                    total: cpu_total.clone(),
                    place,
                    value: |r: &ProcessRow| (percent(r.cpu_percent), r.cpu_percent < Cpu::Zero),
                    heat: move |r: &ProcessRow| {
                        let load = if is_idle(r) { 0.0 } else { r.cpu_percent / 100.0 };
                        (load, accent)
                    },
                    threshold: Cpu::HeatThreshold,
                }),
                ProcessColumn::Memory => metric_column(MetricColumn {
                    id: ProcessColumn::Memory,
                    label: l10n.processes_col_memory(),
                    total: memory_used.clone(),
                    place,
                    value: move |r: &ProcessRow| {
                        let shown = if memory_as_percent {
                            percent(memory_share(r.memory_bytes) * 100.0)
                        } else {
                            format::bytes(units, r.memory_bytes)
                        };
                        (shown, r.memory_bytes == 0)
                    },
                    heat: move |r: &ProcessRow| (memory_share(r.memory_bytes), memory_heat_color(r, accent, palette)),
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Net => metric_column(MetricColumn {
                    id: ProcessColumn::Net,
                    label: l10n.processes_col_net(),
                    total: format::rate(network, net_total),
                    place,
                    value: move |r: &ProcessRow| (format::rate(network, r.net_bytes), r.net_bytes == 0),
                    heat: move |r: &ProcessRow| (r.net_bytes as f32 / net_max, accent),
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Disk => metric_column(MetricColumn {
                    id: ProcessColumn::Disk,
                    label: l10n.processes_col_disk(),
                    total: format::rate(disk, disk_total),
                    place,
                    value: move |r: &ProcessRow| (format::rate(disk, r.disk_bytes), r.disk_bytes == 0),
                    heat: move |r: &ProcessRow| (r.disk_bytes as f32 / disk_max, accent),
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Gpu => metric_column(MetricColumn {
                    id: ProcessColumn::Gpu,
                    label: l10n.processes_col_gpu(),
                    total: gpu_total.clone(),
                    place,
                    value: |r: &ProcessRow| (percent(r.gpu_percent), r.gpu_percent < Cpu::Zero),
                    heat: move |r: &ProcessRow| (r.gpu_percent / 100.0, accent),
                    threshold: Cpu::HeatThreshold,
                }),
                ProcessColumn::GpuMemory => metric_column(MetricColumn {
                    id: ProcessColumn::GpuMemory,
                    label: l10n.processes_col_gpu_memory(),
                    total: gpu_memory_total.clone(),
                    place,
                    value: move |r: &ProcessRow| (format::bytes(units, r.gpu_memory_bytes), r.gpu_memory_bytes == 0),
                    heat: move |r: &ProcessRow| (r.gpu_memory_bytes as f32 / gpu_memory_max, accent),
                    threshold: Heat::Threshold,
                }),
                ProcessColumn::Status => detail_column(column.column, l10n, place, DetailScope::Process, |l10n, r| {
                    status_label(l10n, r.details.status)
                }),
                ProcessColumn::Publisher => detail_column(column.column, l10n, place, DetailScope::Group, |_, r| {
                    r.details.publisher.to_string()
                }),
                ProcessColumn::User => {
                    detail_column(column.column, l10n, place, DetailScope::Group, |_, r| r.details.user.to_string())
                }
                ProcessColumn::CommandLine => detail_column(column.column, l10n, place, DetailScope::Process, |_, r| {
                    r.details.command_line.to_string()
                }),
                ProcessColumn::ImagePath => {
                    detail_column(column.column, l10n, place, DetailScope::Group, |_, r| r.exe_path.to_string())
                }
                ProcessColumn::GpuEngine => {
                    detail_column(column.column, l10n, place, DetailScope::Process, gpu_engine_label)
                }
                ProcessColumn::Platform => detail_column(column.column, l10n, place, DetailScope::Group, |l10n, r| {
                    platform_label(l10n, r.details.architecture)
                }),
                ProcessColumn::Elevated => detail_column(column.column, l10n, place, DetailScope::Process, |l10n, r| {
                    elevated_label(l10n, r.details.elevated)
                }),
                ProcessColumn::Isolation => detail_column(column.column, l10n, place, DetailScope::Group, |l10n, r| {
                    isolation_label(l10n, r.details.isolation)
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
