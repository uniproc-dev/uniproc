use std::time::Duration;

use app_contracts::features::settings::{
    AppTheme, ByteUnits, SetByteUnits, SetStartPage, SetTheme, SetUpdateInterval, SettingsState, StartPage,
    UpdateInterval,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    ChildrenControl, ComboBox, ContentControl, LayoutControl, NumberBox, Orientation, ScrollViewer, Slider,
    StackPanel, Thickness, VerticalAlignment, View,
};

use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::setting_card::{setting_card, SettingCard, SettingCardSize};
use crate::widgets::text::{body_strong, subtitle, text};

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsMark {
    Theme,
    StartPage,
    UpdateSpeed,
    UpdateSpeedValue,
    ByteUnits,
}

struct Control;

#[expect(non_upper_case_globals)]
impl Control {
    const ChoiceWidth: f64 = 180.0;
    const SliderWidth: f64 = 200.0;
    const ValueWidth: f64 = 88.0;
}

struct Layout;

#[expect(non_upper_case_globals)]
impl Layout {
    const MaxWidth: f64 = 1064.0;
    const CardSpacing: f64 = 4.0;

    fn section_header() -> Thickness {
        Thickness::new(1.0, 30.0, 0.0, 6.0)
    }
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

fn section(title: String, cards: impl Into<View>) -> View {
    let header = body_strong(title).margin(Layout::section_header());
    StackPanel::new().children((header, cards)).into()
}

fn theme_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    let choice = ComboBox::new()
        .mark(SettingsMark::Theme)
        .width(Control::ChoiceWidth)
        .items_source(AppTheme::ALL.map(|theme| theme_label(l10n, theme)))
        .selected_index(AppTheme::ALL.iter().position(|theme| *theme == state.theme))
        .on_selection_changed(move |index: Option<usize>| {
            if let Some(theme) = index.and_then(|index| AppTheme::ALL.get(index)) {
                dispatch.emit(SetTheme(*theme));
            }
        });
    setting_card(
        SettingCard {
            icon: Some(icon!(color).size(SettingCardSize::Icon).build()),
            title: l10n.settings_theme(),
            description: l10n.settings_theme_description(),
            control: choice.into(),
        },
        palette,
    )
}

fn start_page_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    let choice = ComboBox::new()
        .mark(SettingsMark::StartPage)
        .width(Control::ChoiceWidth)
        .items_source(StartPage::ALL.map(|page| start_page_label(l10n, page)))
        .selected_index(StartPage::ALL.iter().position(|page| *page == state.start_page))
        .on_selection_changed(move |index: Option<usize>| {
            if let Some(page) = index.and_then(|index| StartPage::ALL.get(index)) {
                dispatch.emit(SetStartPage(*page));
            }
        });
    setting_card(
        SettingCard {
            icon: Some(icon!(start_page).size(SettingCardSize::Icon).build()),
            title: l10n.settings_start_page(),
            description: l10n.settings_start_page_description(),
            control: choice.into(),
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
    let choice = ComboBox::new()
        .mark(SettingsMark::ByteUnits)
        .width(Control::ChoiceWidth)
        .items_source(ByteUnits::ALL.map(|units| byte_units_label(l10n, units)))
        .selected_index(ByteUnits::ALL.iter().position(|units| *units == state.byte_units))
        .on_selection_changed(move |index: Option<usize>| {
            if let Some(units) = index.and_then(|index| ByteUnits::ALL.get(index)) {
                dispatch.emit(SetByteUnits(*units));
            }
        });
    setting_card(
        SettingCard {
            icon: Some(icon!(byte_units).size(SettingCardSize::Icon).build()),
            title: l10n.settings_byte_units(),
            description: l10n.settings_byte_units_description(),
            control: choice.into(),
        },
        palette,
    )
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
            icon: Some(icon!(top_speed).size(SettingCardSize::Icon).build()),
            title: l10n.settings_update_speed(),
            description: l10n.settings_update_speed_description(),
            control: control.into(),
        },
        palette,
    )
}

pub fn settings_view(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let appearance = section(
        l10n.settings_section_appearance(),
        theme_card(state, dispatch, l10n, palette),
    );
    let general = section(
        l10n.settings_section_general(),
        StackPanel::new().spacing(Layout::CardSpacing).children((
            start_page_card(state, dispatch, l10n, palette),
            update_speed_card(state, dispatch, l10n, palette),
            byte_units_card(state, dispatch, l10n, palette),
        )),
    );

    ScrollViewer::new()
        .content(
            StackPanel::new()
                .max_width(Layout::MaxWidth)
                .margin(Thickness::new(space::Page, space::Section, space::Page, space::Page))
                .children((subtitle(l10n.settings_title()), appearance, general)),
        )
        .into()
}
