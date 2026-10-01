use std::time::Duration;

use app_contracts::features::settings::{
    AppTheme, ByteUnits, SetByteUnits, SetStartPage, SetTheme, SetUpdateInterval, SettingsState, ShowSidebarChart,
    SidebarChart, StartPage, UpdateInterval,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, CheckBox, ChildrenControl, ContentControl, Expander, HorizontalAlignment, LayoutControl,
    NumberBox, Orientation, Slider, StackPanel, Thickness, VerticalAlignment, View,
};

use super::marks::SettingsMark;
use crate::l10n::L10n;
use crate::theme::{setting, space, Palette};
use crate::widgets::settings_column::{settings_column, settings_section};
use crate::widgets::setting_card::{card_words, choice, setting_card, SettingCard};
use crate::widgets::text::{subtitle, text};

struct Control;

#[expect(non_upper_case_globals)]
impl Control {
    const SliderWidth: f64 = 200.0;
    const ValueWidth: f64 = 88.0;
}

struct Layout;

#[expect(non_upper_case_globals)]
impl Layout {
    const ExpanderContentInset: f64 = 36.0;
}

fn theme_label(l10n: &L10n, theme: AppTheme) -> String {
    match theme {
        AppTheme::System => l10n.settings_theme_system(),
        AppTheme::Light => l10n.settings_theme_light(),
        AppTheme::Dark => l10n.settings_theme_dark(),
    }
}

fn start_page_label(l10n: &L10n, page: StartPage) -> String {
    match page {
        StartPage::LastOpened => l10n.settings_start_page_last_opened(),
        StartPage::Processes => l10n.settings_start_page_processes(),
        StartPage::Services => l10n.settings_start_page_services(),
        StartPage::Wsl => l10n.settings_start_page_wsl(),
    }
}

fn theme_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    setting_card(
        SettingCard {
            icon: Some(icon!(color).size(setting::Icon).build()),
            title: l10n.settings_theme(),
            description: l10n.settings_theme_description(),
            control: choice(
                SettingsMark::Theme,
                &AppTheme::ALL,
                state.theme,
                |theme| theme_label(l10n, theme),
                move |theme| dispatch.emit(SetTheme(theme)),
            ),
        },
        palette,
    )
}

fn start_page_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    setting_card(
        SettingCard {
            icon: Some(icon!(start_page).size(setting::Icon).build()),
            title: l10n.settings_start_page(),
            description: l10n.settings_start_page_description(),
            control: choice(
                SettingsMark::StartPage,
                &StartPage::ALL,
                state.start_page,
                |page| start_page_label(l10n, page),
                move |page| dispatch.emit(SetStartPage(page)),
            ),
        },
        palette,
    )
}

fn byte_units_label(l10n: &L10n, units: ByteUnits) -> String {
    match units {
        ByteUnits::Windows => l10n.settings_byte_units_windows(),
        ByteUnits::Iec => l10n.settings_byte_units_iec(),
    }
}

fn byte_units_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    setting_card(
        SettingCard {
            icon: Some(icon!(byte_units).size(setting::Icon).build()),
            title: l10n.settings_byte_units(),
            description: l10n.settings_byte_units_description(),
            control: choice(
                SettingsMark::ByteUnits,
                &ByteUnits::ALL,
                state.byte_units,
                |units| byte_units_label(l10n, units),
                move |units| dispatch.emit(SetByteUnits(units)),
            ),
        },
        palette,
    )
}

fn sidebar_chart_label(l10n: &L10n, chart: SidebarChart) -> String {
    match chart {
        SidebarChart::Cpu => l10n.settings_sidebar_chart_cpu(),
        SidebarChart::Memory => l10n.settings_sidebar_chart_memory(),
        SidebarChart::Disk => l10n.settings_sidebar_chart_disk(),
        SidebarChart::Network => l10n.settings_sidebar_chart_network(),
        SidebarChart::Gpu => l10n.settings_sidebar_chart_gpu(),
    }
}

fn sidebar_charts_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let toggles = SidebarChart::ALL.map(|chart| {
        let dispatch = dispatch.clone();
        CheckBox::new()
            .mark(SettingsMark::sidebar_chart(chart))
            .is_checked(state.sidebar_charts.shows(chart))
            .on_is_checked_changed(move |shown: bool| dispatch.emit(ShowSidebarChart(chart, shown)))
            .content(text(sidebar_chart_label(l10n, chart)))
    });
    let header = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Card)
        .margin(Thickness::xy(0.0, setting::ExpanderHeaderInset))
        .children((
            Border::new()
                .vertical_alignment(VerticalAlignment::Center)
                .content(icon!(sidebar_charts).size(setting::Icon).build()),
            card_words(
                l10n.settings_sidebar_charts(),
                Some(l10n.settings_sidebar_charts_description()),
                palette,
            ),
        ));
    Expander::new()
        .mark(SettingsMark::SidebarCharts)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .header(header)
        .content(
            StackPanel::new()
                .margin(Thickness::new(Layout::ExpanderContentInset, 0.0, 0.0, 0.0))
                .children(toggles),
        )
        .into()
}

fn update_speed_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    let millis = |interval: Duration| interval.as_millis() as f64;
    let typed = dispatch.clone();
    let value = NumberBox::new()
        .mark(SettingsMark::UpdateSpeedValue)
        .width(Control::ValueWidth)
        .minimum(millis(UpdateInterval::Min))
        .maximum(millis(UpdateInterval::Max))
        .value(state.update_interval_ms as f64)
        .vertical_alignment(VerticalAlignment::Center)
        .on_value_changed(move |value: Option<f64>| {
            if let Some(value) = value {
                typed.emit(SetUpdateInterval(value.round() as u64));
            }
        });
    let unit = text(l10n.settings_update_speed_unit()).vertical_alignment(VerticalAlignment::Center);
    let slider = Slider::new()
        .mark(SettingsMark::UpdateSpeed)
        .width(Control::SliderWidth)
        .minimum(millis(UpdateInterval::Min))
        .maximum(millis(UpdateInterval::Max))
        .step_frequency(millis(UpdateInterval::Step))
        .value(state.update_interval_ms as f64)
        .vertical_alignment(VerticalAlignment::Center)
        .on_value_changed(move |value: f64| {
            dispatch.emit(SetUpdateInterval(value.round() as u64));
        });
    let control = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Control)
        .children((slider.margin(Thickness::new(0.0, 0.0, space::Header, 0.0)), value, unit));
    setting_card(
        SettingCard {
            icon: Some(icon!(top_speed).size(setting::Icon).build()),
            title: l10n.settings_update_speed(),
            description: l10n.settings_update_speed_description(),
            control: control.into(),
        },
        palette,
    )
}

pub fn settings_view(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let appearance = settings_section(
        l10n.settings_section_appearance(),
        (theme_card(state, dispatch, l10n, palette),),
    );
    let general = settings_section(
        l10n.settings_section_general(),
        (
            start_page_card(state, dispatch, l10n, palette),
            update_speed_card(state, dispatch, l10n, palette),
            byte_units_card(state, dispatch, l10n, palette),
            sidebar_charts_card(state, dispatch, l10n, palette),
        ),
    );

    settings_column((subtitle(l10n.settings_title()), appearance, general))
}
