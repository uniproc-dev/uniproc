use guicons::icon;
use guinea_widgets::resize::{resize_handle, RESIZE_HANDLE_WIDTH};
use std::rc::Rc;

use windows_reactor::{
    keyed, AutoSuggestBox, Border, Callback, Grid, GridLength, HorizontalAlignment, Icon, NavigationView,
    NavigationViewBackButtonVisible, NavigationViewItem, NavigationViewPaneDisplayMode, Thickness, TitleBar,
    VerticalAlignment, View, WindowTitleBarHeight,
};

use crate::l10n::L10n;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShellNav {
    Processes,
    Services,
    Wsl,
    System,
    Settings,
}

impl ShellNav {
    const ALL: [Self; 5] = [Self::Processes, Self::Services, Self::Wsl, Self::System, Self::Settings];

    fn tag(self) -> &'static str {
        match self {
            Self::Processes => "processes",
            Self::Services => "services",
            Self::Wsl => "wsl",
            Self::System => "system",
            Self::Settings => "settings",
        }
    }

    fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|nav| nav.tag() == tag)
    }

    fn in_footer(self) -> bool {
        matches!(self, Self::Settings)
    }

    fn label(self, l10n: &L10n) -> String {
        match self {
            Self::Processes => l10n.shell_nav_processes(),
            Self::Services => l10n.shell_nav_services(),
            Self::Wsl => l10n.shell_nav_wsl(),
            Self::System => l10n.shell_nav_system(),
            Self::Settings => l10n.shell_nav_settings(),
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Processes => icon!(apps_list).build(),
            Self::Services => icon!(puzzle).build(),
            Self::Wsl => icon!(linux).build(),
            Self::System => icon!(system).build(),
            Self::Settings => icon!(settings).build(),
        }
    }
}

pub struct ShellProps<'a> {
    pub l10n: &'a L10n,
    pub open: bool,
    pub width: f64,
    pub menu: Vec<(ShellNav, bool)>,
    pub content: View,
    pub pane_footer: View,
    pub overlay: View,
    pub on_select: Callback<ShellNav>,
    pub on_resize: Callback<f64>,
    pub on_open_changed: Callback<bool>,
}

pub fn shell_view(props: ShellProps<'_>) -> View {
    let title_bar = TitleBar::new()
        .preferred_height(WindowTitleBarHeight::Tall)
        .is_pane_toggle_button_visible(false)
        .grid_row(0)
        .title(props.l10n.shell_window_title())
        .content(
            AutoSuggestBox::new()
                .placeholder_text(props.l10n.shell_search())
                .width(Title::SearchWidth)
                .vertical_alignment(VerticalAlignment::Center),
        );
    let handle = sidebar_resize_handle(&props);
    let overlay = Border::new().grid_row(0).grid_row_span(2).content(props.overlay.clone());

    let mut layers: Vec<View> = vec![title_bar.into(), navigation(&props)];
    layers.extend(handle);
    layers.push(overlay.into());

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .columns([GridLength::Star(1.0)])
        .children(layers)
        .into()
}

fn navigation(props: &ShellProps<'_>) -> View {
    let l10n = props.l10n;
    let to_nav_item = |&(nav, current): &(ShellNav, bool)| {
        keyed(
            nav.tag(),
            NavigationViewItem::new()
                .tag(nav.tag())
                .content(nav.label(l10n))
                .icon(nav.icon())
                .is_selected(current),
        )
    };
    let nav_items: Vec<_> = props.menu.iter().filter(|(nav, _)| !nav.in_footer()).map(to_nav_item).collect();
    let footer_nav_items: Vec<_> = props.menu.iter().filter(|(nav, _)| nav.in_footer()).map(to_nav_item).collect();
    let on_select = props.on_select.clone();

    NavigationView::new()
        .keyed_menu_items(nav_items)
        .keyed_footer_menu_items(footer_nav_items)
        .pane_footer(props.pane_footer.clone())
        .on_selected_tag_changed(move |tag: Option<Rc<str>>| {
            if let Some(nav) = tag.as_deref().and_then(ShellNav::from_tag) {
                on_select.call(nav);
            }
        })
        .is_pane_open(props.open)
        .pane_display_mode(NavigationViewPaneDisplayMode::Left)
        .is_pane_toggle_button_visible(true)
        .on_is_pane_open_changed(props.on_open_changed.clone())
        .is_back_button_visible(NavigationViewBackButtonVisible::Collapsed)
        .is_settings_visible(false)
        .open_pane_length(props.width)
        .grid_row(1)
        .content(props.content.clone())
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
