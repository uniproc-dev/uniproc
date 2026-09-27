use std::time::Duration;

use app_contracts::features::settings::{
    AppTheme, SetStartPage, SetTheme, SetUpdateInterval, SettingsState, StartPage, UpdateInterval,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    ChildrenControl, ComboBox, ContentControl, LayoutControl, Orientation, ScrollViewer, Slider, StackPanel,
    Thickness, VerticalAlignment, View,
};

use super::components::setting_card::{setting_card, SettingCard};
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::text::{body_strong, subtitle, text};

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsMark {
    Theme,
    StartPage,
    UpdateSpeed,
    UpdateSpeedValue,
}

struct Control;

#[expect(non_upper_case_globals)]
impl Control {
    const ChoiceWidth: f64 = 180.0;
    const SliderWidth: f64 = 200.0;
    const ValueWidth: f64 = 56.0;
}

pub fn update_interval_label(l10n: &L10n, interval: Duration) -> String {
    let ms = interval.as_millis();
    if ms < 1000 {
        l10n.settings_update_speed_ms(ms as i64)
    } else {
        let seconds = format!("{:.1}", interval.as_secs_f64());
        l10n.settings_update_speed_seconds(seconds.trim_end_matches(".0").to_string())
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
    let header = body_strong(title).margin(Thickness::new(0.0, space::Section, 0.0, space::Control));
    StackPanel::new().spacing(space::Compact).children((header, cards)).into()
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
            icon: icon!(color).size(size::CommandIcon).build(),
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
            icon: icon!(start_page).size(size::CommandIcon).build(),
            title: l10n.settings_start_page(),
            description: l10n.settings_start_page_description(),
            control: choice.into(),
        },
        palette,
    )
}

fn update_speed_card(state: &SettingsState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let dispatch = dispatch.clone();
    let millis = |interval: Duration| interval.as_millis() as f64;
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
    let value = text(update_interval_label(l10n, Duration::from_millis(state.update_interval_ms)))
        .mark(SettingsMark::UpdateSpeedValue)
        .width(Control::ValueWidth)
        .vertical_alignment(VerticalAlignment::Center);
    let control = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Header)
        .children((slider, value));
    setting_card(
        SettingCard {
            icon: icon!(top_speed).size(size::CommandIcon).build(),
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
        StackPanel::new().spacing(space::Compact).children((
            start_page_card(state, dispatch, l10n, palette),
            update_speed_card(state, dispatch, l10n, palette),
        )),
    );

    ScrollViewer::new()
        .content(
            StackPanel::new()
                .margin(Thickness::xy(space::Section, space::Card))
                .children((subtitle(l10n.settings_title()), appearance, general)),
        )
        .into()
}
