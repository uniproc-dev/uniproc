use app_contracts::features::activity::{
    ActivityState, ApplyPreset, DeletePreset, Filter, Preset, SavePreset, UnhideInPreset,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    keyed, Border, Callback, Grid, GridLength, HorizontalAlignment, KeyedView, Orientation, StackPanel, TextBox, ThemeBrush,
    Thickness, VerticalAlignment, View,
};

use super::components::picks::named;
use super::marks::ActivityPresetsMark;
use crate::l10n::L10n;
use crate::theme::{radius, setting, size, space, Palette};
use crate::widgets::breadcrumb::{breadcrumb, Breadcrumb};
use crate::widgets::button::{action_button, command_button};
use crate::widgets::card::card;
use crate::widgets::setting_card::{setting_card, SettingCard};
use crate::widgets::settings_column::{settings_column, settings_section};
use crate::widgets::text::{body_strong, caption, text};

struct NameBox;

#[expect(non_upper_case_globals)]
impl NameBox {
    const Width: f64 = 220.0;
}

#[derive(Clone)]
pub enum ActivityPresetsMsg {
    Name(String),
    Save,
    BackHovered(bool),
}

#[derive(Default)]
pub struct ActivityPresetsPage {
    name: String,
    back_hovered: bool,
}

fn summary(filter: &Filter, l10n: &L10n) -> Vec<String> {
    let mut lines = Vec::new();
    if !filter.came {
        lines.push(l10n.activity_presets_came_off());
    }
    if !filter.went {
        lines.push(l10n.activity_presets_went_off());
    }
    if filter.new_only {
        lines.push(l10n.activity_presets_new_only());
    }
    if !filter.series {
        lines.push(l10n.activity_presets_series_off());
    }
    if let Some(only) = &filter.only {
        lines.push(l10n.activity_presets_only(named(only)));
    }
    if lines.is_empty() && filter.hidden.is_empty() {
        lines.push(l10n.activity_presets_shows_everything());
    }
    lines
}

impl ActivityPresetsPage {
    pub fn update(&mut self, message: ActivityPresetsMsg, dispatch: &Dispatch) {
        match message {
            ActivityPresetsMsg::Name(name) => self.name = name,
            ActivityPresetsMsg::Save => {
                let name = self.name.trim();
                if !name.is_empty() {
                    dispatch.emit(SavePreset(name.to_string()));
                    self.name.clear();
                }
            }
            ActivityPresetsMsg::BackHovered(hovered) => self.back_hovered = hovered,
        }
    }

    fn save_card(&self, l10n: &L10n, palette: Palette, forward: &Callback<ActivityPresetsMsg>) -> View {
        let (named, saved) = (forward.clone(), forward.clone());
        let control = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Control)
            .children((
                TextBox::new(&self.name)
                    .mark(ActivityPresetsMark::Name)
                    .width(NameBox::Width)
                    .placeholder_text(l10n.activity_presets_name())
                    .on_text_changed(move |name: std::rc::Rc<str>| named.call(ActivityPresetsMsg::Name(name.to_string()))),
                action_button(
                    ActivityPresetsMark::Save,
                    l10n.activity_presets_save_button(),
                    None,
                    !self.name.trim().is_empty(),
                    move || saved.call(ActivityPresetsMsg::Save),
                ),
            ));
        setting_card(
            SettingCard {
                icon: Some(icon!(save).size(setting::Icon).build_element()),
                title: l10n.activity_presets_save(),
                description: l10n.activity_presets_save_description(),
                control: control.into(),
            },
            palette,
        )
    }

    fn preset_card(preset: &Preset, current: bool, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
        let mut words: Vec<KeyedView> = vec![keyed("name", body_strong(preset.name.clone()))];
        for (index, line) in summary(&preset.filter, l10n).into_iter().enumerate() {
            words.push(keyed(format!("line-{index}"), caption(line).foreground(palette.secondary_text)));
        }
        for pick in &preset.filter.hidden {
            let (dispatch, preset, pick) = (dispatch.clone(), preset.name.clone(), pick.clone());
            let label = l10n.activity_presets_hidden(named(&pick));
            words.push(keyed(
                pick.id(),
                Border::new().horizontal_alignment(HorizontalAlignment::Left).content(command_button(
                    ActivityPresetsMark::Hidden,
                    label,
                    Some(icon!(dismiss).size(size::Icon).build_element()),
                    true,
                    move || {
                        dispatch.emit(UnhideInPreset {
                            preset: preset.clone(),
                            pick: pick.clone(),
                        })
                    },
                )),
            ));
        }

        let use_it: View = if current {
            caption(l10n.activity_presets_current())
                .mark(ActivityPresetsMark::Current)
                .foreground(palette.secondary_text)
                .vertical_alignment(VerticalAlignment::Center)
                .into()
        } else {
            let (dispatch, name) = (dispatch.clone(), preset.name.clone());
            action_button(ActivityPresetsMark::Apply, l10n.activity_presets_apply(), None, true, move || {
                dispatch.emit(ApplyPreset(name.clone()))
            })
        };
        let (dispatch, name) = (dispatch.clone(), preset.name.clone());
        let delete = command_button(
            ActivityPresetsMark::Delete,
            l10n.activity_presets_delete(),
            Some(icon!(delete).size(size::Icon).build_element()),
            true,
            move || dispatch.emit(DeletePreset(name.clone())),
        );

        card()
            .corner_radius(radius::Control)
            .border_thickness(Thickness::uniform(space::Hairline))
            .border_brush(ThemeBrush::CardStroke)
            .padding(Thickness::uniform(space::Card))
            .content(
                Grid::new()
                    .columns([GridLength::Star(1.0), GridLength::Auto])
                    .children((
                        StackPanel::new().grid_column(0).keyed_children(words),
                        StackPanel::new()
                            .grid_column(1)
                            .orientation(Orientation::Horizontal)
                            .spacing(space::Control)
                            .vertical_alignment(VerticalAlignment::Top)
                            .children((use_it, delete)),
                    )),
            )
            .into()
    }

    pub fn view(
        &self,
        state: &ActivityState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ActivityPresetsMsg>,
        back: Callback<()>,
    ) -> View {
        let hover = forward.clone();
        let crumbs = breadcrumb(
            ActivityPresetsMark::Back,
            Breadcrumb {
                parent: l10n.activity_title(),
                current: l10n.activity_presets_title(),
                hovered: self.back_hovered,
            },
            palette,
            Callback::new(move |hovered| hover.call(ActivityPresetsMsg::BackHovered(hovered))),
            back,
        );

        let current = state.preset().map(|preset| preset.name.clone());
        let saved: Vec<View> = if state.presets.is_empty() {
            vec![text(l10n.activity_presets_empty())
                .mark(ActivityPresetsMark::Empty)
                .foreground(palette.secondary_text)
                .margin(Thickness::new(setting::CaptionInset, 0.0, 0.0, 0.0))
                .into()]
        } else {
            state
                .presets
                .iter()
                .map(|preset| {
                    Self::preset_card(preset, current.as_deref() == Some(&preset.name), dispatch, l10n, palette)
                })
                .collect()
        };

        settings_column((
            crumbs,
            StackPanel::new()
                .margin(Thickness::new(0.0, space::Section, 0.0, 0.0))
                .children((
                    self.save_card(l10n, palette, &forward),
                    settings_section(l10n.activity_presets_saved(), saved),
                )),
        ))
    }
}
