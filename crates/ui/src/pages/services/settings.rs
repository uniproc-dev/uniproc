use app_contracts::features::services::ServiceColumn;
use guicons::icon;
use guinea::Mark;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, Grid, GridLength, Orientation, ResourceOverrides,
    StackPanel, Thickness, VerticalAlignment, View,
};

use super::ServicesSettingsMark;
use super::components::column_layout::{ColumnLayout, ServicesSettingsMaps};
use super::components::columns::column_label;
use crate::l10n::L10n;
use crate::theme::{Palette, setting, size, space};
use crate::widgets::breadcrumb::{Breadcrumb, breadcrumb};
use crate::widgets::button::action_button;
use crate::widgets::setting_card::{card_words, expander_rows, setting_expander, setting_switch};
use crate::widgets::settings_column::settings_column;
use crate::widgets::text::text;

#[derive(Clone, Copy)]
pub enum ServicesSettingsMsg {
    ShowColumn(ServiceColumn, bool),
    MoveColumn(ServiceColumn, bool),
    ResetColumns,
    Expand(bool),
    BackHovered(bool),
}

pub struct ServicesSettingsPage {
    layout: ColumnLayout,
    expanded: bool,
    back_hovered: bool,
}

impl ServicesSettingsPage {
    pub fn new(settings: ServicesSettingsMaps) -> Self {
        Self {
            layout: ColumnLayout::new(Some(settings)),
            expanded: false,
            back_hovered: false,
        }
    }

    pub fn update(&mut self, message: ServicesSettingsMsg) {
        match message {
            ServicesSettingsMsg::ShowColumn(column, visible) => self.layout.show(column, visible),
            ServicesSettingsMsg::MoveColumn(column, down) => self.layout.shift(column, down),
            ServicesSettingsMsg::ResetColumns => self.layout.reset(),
            ServicesSettingsMsg::Expand(expanded) => self.expanded = expanded,
            ServicesSettingsMsg::BackHovered(hovered) => self.back_hovered = hovered,
        }
    }

    pub fn view(
        &self,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ServicesSettingsMsg>,
        back: Callback<()>,
    ) -> View {
        let hovered = forward.clone();
        let crumbs = breadcrumb(
            ServicesSettingsMark::Back,
            Breadcrumb {
                parent: l10n.services_title(),
                current: l10n.services_settings_title(),
                hovered: self.back_hovered,
            },
            palette,
            Callback::new(move |on| hovered.call(ServicesSettingsMsg::BackHovered(on))),
            back,
        );
        let placed = self.layout.placed();
        let mut rows: Vec<(String, View)> = placed
            .iter()
            .enumerate()
            .map(|(at, &column)| {
                let visible = self.layout.config(column).visible;
                let shown = forward.clone();
                let switch = setting_switch(
                    ServicesSettingsMark::Shown,
                    visible,
                    column != ServiceColumn::Name,
                    if visible {
                        l10n.services_settings_shown_on()
                    } else {
                        l10n.services_settings_shown_off()
                    },
                    palette,
                    move |on| shown.call(ServicesSettingsMsg::ShowColumn(column, on)),
                );
                let button = |down, mark, enabled, image: View| {
                    let forward = forward.clone();
                    Button::new()
                        .mark(mark)
                        .style(ButtonStyle::Subtle)
                        .is_enabled(enabled)
                        .resource_overrides(
                            ResourceOverrides::new()
                                .set("ButtonPadding", Thickness::uniform(space::Compact)),
                        )
                        .on_click(move || {
                            forward.call(ServicesSettingsMsg::MoveColumn(column, down))
                        })
                        .content(image)
                };
                let moves = StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(space::Compact)
                    .grid_column(1)
                    .vertical_alignment(VerticalAlignment::Center)
                    .children((
                        button(
                            false,
                            ServicesSettingsMark::Up,
                            at > 1,
                            icon!(chevron_up_regular).size(size::Icon).build_element(),
                        ),
                        button(
                            true,
                            ServicesSettingsMark::Down,
                            at > 0 && at + 1 < placed.len(),
                            icon!(chevron_down_regular).size(size::Icon).build_element(),
                        ),
                    ));
                (
                    column.name().to_string(),
                    Border::new()
                        .mark(column)
                        .content(
                            Grid::new()
                                .columns([
                                    GridLength::Star(1.0),
                                    GridLength::Auto,
                                    GridLength::Auto,
                                ])
                                .column_spacing(space::Card)
                                .children((
                                    text(column_label(l10n, column))
                                        .vertical_alignment(VerticalAlignment::Center),
                                    moves,
                                    Grid::new()
                                        .grid_column(2)
                                        .vertical_alignment(VerticalAlignment::Center)
                                        .children((switch,)),
                                )),
                        )
                        .into(),
                )
            })
            .collect();
        let reset = forward.clone();
        rows.push((
            "reset".into(),
            Grid::new()
                .columns([GridLength::Star(1.0), GridLength::Auto])
                .children((
                    text(l10n.services_settings_columns_reset())
                        .vertical_alignment(VerticalAlignment::Center),
                    Grid::new().grid_column(1).children((action_button(
                        ServicesSettingsMark::ResetColumns,
                        l10n.services_settings_reset(),
                        None,
                        !self.layout.is_default(),
                        move || reset.call(ServicesSettingsMsg::ResetColumns),
                    ),)),
                ))
                .into(),
        ));
        let expanded = forward;
        let columns = setting_expander()
            .mark(ServicesSettingsMark::ColumnsGroup)
            .is_expanded(self.expanded)
            .on_is_expanded_changed(move |on| expanded.call(ServicesSettingsMsg::Expand(on)))
            .header(
                Border::new()
                    .margin(Thickness::xy(0.0, setting::ExpanderHeaderInset))
                    .content(card_words(
                        l10n.services_settings_columns(),
                        Some(l10n.services_settings_columns_description()),
                        palette,
                    )),
            )
            .content(expander_rows(rows));
        settings_column((
            crumbs,
            StackPanel::new()
                .margin(Thickness::new(0.0, space::Section, 0.0, 0.0))
                .children((columns,)),
        ))
    }
}
