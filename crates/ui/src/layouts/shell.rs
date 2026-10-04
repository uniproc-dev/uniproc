use app_contracts::features::metrics::MetricsState;
use app_contracts::features::settings::{SidebarChart, SidebarCharts, Units};
use guicons::icon;
use guinea_widgets::chart::Chart;
use guinea_widgets::resize::{resize_handle, RESIZE_HANDLE_WIDTH};
use std::rc::Rc;

use windows_reactor::{
    keyed, AutoSuggestBox, Border, Callback, Grid, GridLength, HorizontalAlignment, Icon, NavigationView,
    NavigationViewBackButtonVisible, NavigationViewItem, NavigationViewPaneDisplayMode, Thickness, TitleBar,
    VerticalAlignment, View, WindowTitleBarHeight,
};

use super::metrics_pane::metrics_pane;
use crate::l10n::L10n;
use crate::theme::{size, Palette};

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
    pub units: Units,
    pub cadence_ms: u64,
    pub charts: &'a [Chart; 5],
    pub shown: SidebarCharts,
    pub on_show_chart: Callback<(SidebarChart, bool)>,
    pub on_select: Callback<Option<String>>,
    pub on_resize: Callback<f64>,
    pub on_open_changed: Callback<bool>,
}

fn nav_items(l10n: &L10n) -> Vec<(&'static str, String, Icon)> {
    vec![
        ("processes", l10n.shell_nav_processes(), icon!(apps_list).size(size::NavIcon).build()),
        ("activity", l10n.shell_nav_activity(), icon!(activity).size(size::NavIcon).build()),
        ("services", l10n.shell_nav_services(), icon!(puzzle).size(size::NavIcon).build()),
        ("wsl", l10n.shell_nav_wsl(), icon!(linux).size(size::NavIcon).build()),
        ("system", l10n.shell_nav_system(), icon!(system).size(size::NavIcon).build()),
    ]
}

fn footer_nav_items(l10n: &L10n) -> Vec<(&'static str, String, Icon)> {
    vec![("settings", l10n.shell_nav_settings(), icon!(settings).size(size::NavIcon).build())]
}

pub fn shell_view(mut props: ShellProps<'_>) -> View {
    let title_bar = TitleBar::new()
        .preferred_height(WindowTitleBarHeight::Tall)
        .is_pane_toggle_button_visible(false)
        .grid_row(0);

    let layers: Vec<View> = match props.splash.take() {
        Some(splash) => vec![
            Border::new()
                .grid_row(0)
                .grid_row_span(2)
                .content(splash)
                .into(),
            title_bar.into(),
        ],
        None => {
            let title_bar = title_bar.title(props.l10n.shell_window_title()).content(
                AutoSuggestBox::new()
                    .placeholder_text(props.l10n.shell_search())
                    .width(Title::SearchWidth)
                    .vertical_alignment(VerticalAlignment::Center),
            );
            let handle = sidebar_resize_handle(&props);
            let mut layers = vec![title_bar.into(), navigation(props)];
            layers.extend(handle);
            layers
        }
    };

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .columns([GridLength::Star(1.0)])
        .children(layers)
        .into()
}

fn navigation(props: ShellProps<'_>) -> View {
    let l10n = props.l10n;
    let selected_tag = props.selected_tag;
    let to_nav_item = |(tag, label, icon): (&'static str, String, Icon)| {
        keyed(
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
    let on_select = props.on_select.clone();

    NavigationView::new()
        .keyed_menu_items(nav_items)
        .keyed_footer_menu_items(footer_nav_items)
        .pane_footer(metrics_pane(&props))
        .on_selected_tag_changed(move |tag: Option<Rc<str>>| on_select.call(tag.map(|tag| tag.to_string())))
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

fn sidebar_resize_handle(props: &ShellProps<'_>) -> Option<View> {
    props.open.then(|| {
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
    })
}
