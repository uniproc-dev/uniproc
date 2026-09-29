use app_contracts::features::metrics::MetricsState;
use app_contracts::features::settings::ByteUnits;
use guicons::icon;
use guinea_widgets::chart::Chart;
use guinea_widgets::resize::{resize_handle, RESIZE_HANDLE_WIDTH};
use windows_reactor::{
    AutoSuggestBox, Border, Callback, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength,
    HorizontalAlignment, LayoutControl, NavigationView, NavigationViewBackButtonVisible,
    NavigationViewItem, NavigationViewPaneDisplayMode, StackPanel, Thickness, TitleBar,
    VerticalAlignment, View, WindowTitleBarHeight,
};

use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::metric_chart::{
    metric_chart, metric_mini_bar, MetricChart, MetricChartKind,
};
use crate::widgets::separator;

struct Title;

#[expect(non_upper_case_globals)]
impl Title {
    const SearchWidth: f64 = 240.0;
}

struct Sidebar;

#[expect(non_upper_case_globals)]
impl Sidebar {
    const MinWidth: f64 = 200.0;
    const MaxWidth: f64 = 500.0;
    const MetricHeight: f64 = 48.0;
}

pub struct ShellProps<'a> {
    pub l10n: &'a L10n,
    pub palette: Palette,
    pub open: bool,
    pub width: f64,
    pub selected_tag: &'a str,
    pub content: View,
    pub splash: Option<View>,
    pub metrics: &'a MetricsState,
    pub units: ByteUnits,
    pub cpu_chart: &'a Chart,
    pub memory_chart: &'a Chart,
    pub on_select: Callback<Option<String>>,
    pub on_resize: Callback<f64>,
    pub on_open_changed: Callback<bool>,
}

fn nav_items(l10n: &L10n) -> Vec<(&'static str, String, View)> {
    vec![
        (
            "processes",
            l10n.shell_nav_processes(),
            icon!(apps_list).size(size::NavIcon).build(),
        ),
        (
            "services",
            l10n.shell_nav_services(),
            icon!(puzzle).size(size::NavIcon).build(),
        ),
        (
            "wsl",
            l10n.shell_nav_wsl(),
            icon!(linux).size(size::NavIcon).build(),
        ),
    ]
}

fn footer_nav_items(l10n: &L10n) -> Vec<(&'static str, String, View)> {
    vec![(
        "settings",
        l10n.shell_nav_settings(),
        icon!(settings).size(size::NavIcon).build(),
    )]
}

fn metrics_pane_footer(props: &ShellProps<'_>) -> View {
    let metrics = props.metrics;
    let palette = props.palette;

    if !props.open {
        let cpu = metric_mini_bar(palette, MetricChartKind::Cpu, &metrics.cpu_history);
        let memory = metric_mini_bar(palette, MetricChartKind::Memory, &metrics.memory_history);
        return Border::new()
            .padding(Thickness::xy(size::NavIcon, space::Control))
            .horizontal_alignment(HorizontalAlignment::Center)
            .content(
                StackPanel::new()
                    .spacing(space::Control)
                    .children((separator(palette), cpu, memory, separator(palette))),
            );
    }

    let machine = metrics.machine.ready();
    let cpu_detail = machine.map(|m| {
        format!(
            "{:.1} / {:.1} GHz",
            m.cpu_current_mhz as f64 / 1000.0,
            m.cpu_max_mhz as f64 / 1000.0
        )
    });
    let memory_detail = machine.map(|m| format::bytes(props.units, m.memory_total_bytes));

    let cpu_chart = metric_chart(MetricChart {
        chart: props.cpu_chart,
        kind: MetricChartKind::Cpu,
        history: &metrics.cpu_history,
        height: Sidebar::MetricHeight,
        detail: cpu_detail,
        palette,
    });
    let memory_chart = metric_chart(MetricChart {
        chart: props.memory_chart,
        kind: MetricChartKind::Memory,
        history: &metrics.memory_history,
        height: Sidebar::MetricHeight,
        detail: memory_detail,
        palette,
    });

    Border::new()
        .padding(Thickness::xy(space::Compact, space::Control))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .content(StackPanel::new().spacing(space::Compact).children((cpu_chart, memory_chart)))
}

pub fn shell_view(mut props: ShellProps<'_>) -> View {
    let title_bar = TitleBar::new()
        .preferred_height(WindowTitleBarHeight::Tall)
        .is_pane_toggle_button_visible(false)
        .grid_row(0);

    let (backdrop, title_bar, body, handle): (View, TitleBar, View, View) = match props.splash.take() {
        Some(splash) => (
            Border::new()
                .grid_row(0)
                .grid_row_span(2)
                .content(splash)
                .into(),
            title_bar,
            View::empty(),
            View::empty(),
        ),
        None => {
            let title_bar = title_bar.title(props.l10n.shell_window_title()).content(
                AutoSuggestBox::new()
                    .placeholder_text(props.l10n.shell_search())
                    .width(Title::SearchWidth)
                    .vertical_alignment(VerticalAlignment::Center),
            );
            let handle = sidebar_resize_handle(&props);
            (View::empty(), title_bar, navigation(props), handle)
        }
    };

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .columns([GridLength::Star(1.0)])
        .children((backdrop, title_bar, body, handle))
        .into()
}

fn navigation(props: ShellProps<'_>) -> View {
    let l10n = props.l10n;
    let selected_tag = props.selected_tag;
    let to_nav_item = |(tag, label, icon): (&'static str, String, View)| {
        (
            tag,
            NavigationViewItem::new()
                .tag(tag)
                .content(label)
                .icon(icon)
                .is_selected(tag == selected_tag),
        )
    };
    let nav_items: Vec<_> = nav_items(l10n).into_iter().map(to_nav_item).collect();
    let footer_nav_items: Vec<_> = footer_nav_items(l10n).into_iter().map(to_nav_item).collect();

    NavigationView::new()
        .menu_items(nav_items)
        .footer_menu_items(footer_nav_items)
        .pane_footer(metrics_pane_footer(&props))
        .on_selected_tag_changed(props.on_select.clone())
        .is_pane_open(props.open)
        .pane_display_mode(NavigationViewPaneDisplayMode::Left)
        .is_pane_toggle_button_visible(true)
        .on_is_pane_open_changed(props.on_open_changed.clone())
        .is_back_button_visible(NavigationViewBackButtonVisible::Collapsed)
        .is_settings_visible(false)
        .open_pane_length(props.width)
        .grid_row(1)
        .content(props.content)
        .into()
}

fn sidebar_resize_handle(props: &ShellProps<'_>) -> View {
    if props.open {
        Border::new()
            .margin(Thickness::new(
                props.width - RESIZE_HANDLE_WIDTH / 2.0,
                0.0,
                0.0,
                0.0,
            ))
            .horizontal_alignment(HorizontalAlignment::Left)
            .grid_row(1)
            .content(
                resize_handle(props.width, props.on_resize.clone())
                    .min(Sidebar::MinWidth)
                    .max(Sidebar::MaxWidth)
                    .build(),
            )
            .into()
    } else {
        View::empty()
    }
}
