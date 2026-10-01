use app_contracts::features::settings::ByteUnits;
use app_contracts::features::wsl::{AgentPresence, DistroRow, LinuxMachineSummary};
use guinea_widgets::table::ColumnSpec;
use windows_reactor::{
    Border, ChildrenControl, Color, ContentControl, LayoutControl, Orientation, StackPanel,
    Thickness, VerticalAlignment, View,
};

use crate::format;
use crate::l10n::L10n;
use crate::theme::{accent_color, size, space, Palette};
use crate::widgets::distro_icon::distro_icon;
use crate::widgets::table_cell;
use crate::widgets::text::text;

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WslColumn {
    Name,
    Status,
    Cpu,
    Memory,
    Net,
    Disk,
}

type Column = ColumnSpec<DistroRow, WslColumn>;

struct NameHeader;

#[expect(non_upper_case_globals)]
impl NameHeader {
    const TextInset: f64 = size::Dot + size::Icon + space::Control * 2.0 + 6.0;
}

struct AgentDot;

#[expect(non_upper_case_globals)]
impl AgentDot {
    const NotChecked: Color = Color::argb(90, 128, 128, 128);
}

fn dimmed(content: impl Into<View>, running: bool) -> View {
    table_cell::dimmed(content, !running)
}

fn agent_dot(presence: AgentPresence, palette: Palette) -> View {
    let color = match presence {
        AgentPresence::Answering => palette.success,
        AgentPresence::Silent => palette.caution,
        AgentPresence::NotChecked => AgentDot::NotChecked,
    };

    Border::new()
        .width(size::Dot)
        .height(size::Dot)
        .corner_radius(size::Dot / 2.0)
        .background(color)
        .vertical_alignment(VerticalAlignment::Center)
        .content(View::empty())
}

fn name_header(l10n: &L10n) -> View {
    Border::new()
        .padding(Thickness::new(NameHeader::TextInset, 0.0, 0.0, 0.0))
        .content(text(l10n.wsl_col_distribution()))
}

fn metric_column(
    id: WslColumn,
    header: String,
    width: f64,
    palette: Palette,
    read: impl Fn(&LinuxMachineSummary) -> (String, f32) + 'static,
) -> Column {
    ColumnSpec::new(id, header, width, move |row: &DistroRow| {
        let Some(metrics) = row.metrics.as_ref() else {
            return dimmed(
                table_cell::cell_text("-").foreground(palette.secondary_text),
                row.running,
            );
        };
        let (text, intensity) = read(metrics);
        table_cell::heat_cell(text, intensity, accent_color())
    })
}

fn name_column(l10n: &L10n, palette: Palette) -> Column {
    let header_l10n = l10n.clone();
    ColumnSpec::new_with_header(
        WslColumn::Name,
        move || name_header(&header_l10n),
        260.0,
        move |row: &DistroRow| {
            let content = StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((
                    agent_dot(row.agent, palette),
                    distro_icon(&row.name),
                    table_cell::cell_text(row.name.clone()),
                ));
            dimmed(content, row.running)
        },
    )
    .flush()
}

fn status_column(l10n: &L10n, palette: Palette) -> Column {
    let cell_l10n = l10n.clone();
    ColumnSpec::new(
        WslColumn::Status,
        l10n.wsl_col_status(),
        110.0,
        move |row: &DistroRow| {
            let (label, color) = if row.running {
                (cell_l10n.wsl_running(), palette.success)
            } else {
                (cell_l10n.wsl_stopped(), palette.secondary_text)
            };
            dimmed(table_cell::cell_text(label).foreground(color), row.running)
        },
    )
}

pub(crate) fn build_columns(l10n: &L10n, palette: Palette, units: ByteUnits) -> Vec<Column> {
    vec![
        name_column(l10n, palette),
        status_column(l10n, palette),
        metric_column(WslColumn::Cpu, l10n.wsl_col_cpu(), 110.0, palette, |m| {
            (
                m.cpu_percent
                    .map(format::percent)
                    .unwrap_or_else(|| "-".into()),
                m.cpu_percent.unwrap_or(0.0) / 100.0,
            )
        }),
        metric_column(WslColumn::Memory, l10n.wsl_col_memory(), 130.0, palette, move |m| {
            let share = if m.memory_total_bytes > 0 {
                m.memory_used_bytes as f32 / m.memory_total_bytes as f32
            } else {
                0.0
            };
            (format::bytes(units, m.memory_used_bytes), share)
        }),
        metric_column(WslColumn::Net, l10n.wsl_col_net(), 110.0, palette, move |m| {
            (format::bytes(units, m.net_bytes), 0.0)
        }),
        metric_column(WslColumn::Disk, l10n.wsl_col_disk(), 110.0, palette, move |m| {
            (format::bytes(units, m.disk_bytes), 0.0)
        }),
    ]
}
