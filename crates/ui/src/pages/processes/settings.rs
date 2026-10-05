use amethystate::{Field, ReactiveMap};
use app_contracts::OrWarn;
use app_contracts::features::processes::ProcessColumn;
use guicons::icon;
use guinea::Mark;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, Grid, GridLength, Orientation, StackPanel, Thickness,
    VerticalAlignment, View,
};

use super::components::Step;
use super::components::column_layout::ColumnLayout;
use super::components::columns::{column_label, section_label};
use super::components::grouping::{SectionId, SectionOrder};
use super::marks::ProcessesSettingsMark;
use super::page::ProcessesSettingsMaps;
use crate::l10n::L10n;
use crate::theme::{Palette, setting, size, space};
use crate::widgets::breadcrumb::{Breadcrumb, breadcrumb};
use crate::widgets::button::action_button;
use crate::widgets::setting_card::{
    SettingCard, card_words, choice, expander_rows, setting_card, setting_expander, setting_switch,
};
use crate::widgets::settings_column::settings_column;
use crate::widgets::text::text;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Columns,
    Sections,
}

impl Group {
    fn mark(self) -> ProcessesSettingsMark {
        match self {
            Self::Columns => ProcessesSettingsMark::ColumnsGroup,
            Self::Sections => ProcessesSettingsMark::SectionsGroup,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ProcessesSettingsMsg {
    ShowColumn(ProcessColumn, bool),
    MoveColumn(ProcessColumn, Step),
    MoveSection(SectionId, Step),
    ResetColumns,
    ResetSections,
    MemoryAsPercent(bool),
    Expand(Group, bool),
    BackHovered(bool),
}

pub struct ProcessesSettingsPage {
    back_hovered: bool,
    open: Vec<Group>,
    layout: ColumnLayout,
    sections: SectionOrder,
    section_ranks: Option<ReactiveMap<String, u32>>,
    memory_as_percent: bool,
    memory_setting: Option<Field<bool>>,
}

impl Default for ProcessesSettingsPage {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ProcessesSettingsPage {
    pub fn new(settings: Option<ProcessesSettingsMaps>) -> Self {
        let (columns, column_order, section_ranks, memory_setting) = match settings {
            Some(maps) => (
                Some(maps.columns),
                Some(maps.column_order),
                Some(maps.section_order),
                Some(maps.memory_as_percent),
            ),
            None => (None, None, None, None),
        };
        Self {
            back_hovered: false,
            open: Vec::new(),
            layout: ColumnLayout::new(columns, column_order),
            sections: SectionOrder::kept(section_ranks.as_ref()),
            section_ranks,
            memory_as_percent: memory_setting.as_ref().is_some_and(Field::get),
            memory_setting,
        }
    }

    pub fn update(&mut self, message: ProcessesSettingsMsg) {
        match message {
            ProcessesSettingsMsg::ShowColumn(column, visible) => self.layout.show(column, visible),
            ProcessesSettingsMsg::MoveColumn(column, step) => self.layout.shift(column, step),
            ProcessesSettingsMsg::MoveSection(section, step) => {
                if let Some(sections) = self.sections.shifted(section, step) {
                    self.sections = sections;
                    if let Some(ranks) = &self.section_ranks {
                        self.sections.store(ranks);
                    }
                }
            }
            ProcessesSettingsMsg::ResetColumns => self.layout.reset(),
            ProcessesSettingsMsg::ResetSections => {
                self.sections = SectionOrder::default();
                if let Some(ranks) = &self.section_ranks {
                    SectionOrder::forget(ranks);
                }
            }
            ProcessesSettingsMsg::MemoryAsPercent(percent) => {
                self.memory_as_percent = percent;
                if let Some(setting) = &self.memory_setting {
                    setting
                        .set(percent)
                        .or_warn("could not keep how memory is shown");
                }
            }
            ProcessesSettingsMsg::Expand(group, open) => {
                self.open.retain(|kept| *kept != group);
                if open {
                    self.open.push(group);
                }
            }
            ProcessesSettingsMsg::BackHovered(hovered) => self.back_hovered = hovered,
        }
    }

    fn heading(&self, group: Group, title: String, description: String) -> Heading {
        Heading {
            group,
            title,
            description,
            open: self.open.contains(&group),
        }
    }

    fn memory_card(
        &self,
        l10n: &L10n,
        palette: Palette,
        forward: &Callback<ProcessesSettingsMsg>,
    ) -> View {
        let forward = forward.clone();
        let choice = choice(
            ProcessesSettingsMark::MemoryValues,
            &[false, true],
            self.memory_as_percent,
            |percent| match percent {
                false => l10n.processes_settings_memory_values(),
                true => l10n.processes_settings_memory_percents(),
            },
            move |percent| {
                forward.call(ProcessesSettingsMsg::MemoryAsPercent(percent));
            },
        );
        setting_card(
            SettingCard {
                icon: None,
                title: l10n.processes_settings_memory(),
                description: l10n.processes_settings_memory_description(),
                control: choice,
            },
            palette,
        )
    }

    pub fn view(
        &self,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesSettingsMsg>,
        back: Callback<()>,
    ) -> View {
        let columns = self.layout.placed().into_iter().map(|column| {
            let moves = move_buttons(
                |step| self.layout.can_shift(column, step),
                &forward,
                move |step| ProcessesSettingsMsg::MoveColumn(column, step),
            );
            let switch = shown_switch(column, self.layout.visible(column), l10n, palette, &forward);
            row(column, column_label(l10n, column), (moves, Some(switch)))
        });

        let sections = self.sections.ids().iter().map(|&section| {
            let moves = move_buttons(
                |step| self.sections.can_shift(section, step),
                &forward,
                move |step| ProcessesSettingsMsg::MoveSection(section, step),
            );
            row(section, section_label(l10n, section), (moves, None))
        });

        let columns = expander(
            self.heading(
                Group::Columns,
                l10n.processes_settings_columns(),
                l10n.processes_settings_columns_description(),
            ),
            columns.collect(),
            reset_row(
                l10n.processes_settings_columns_reset(),
                ProcessesSettingsMark::ResetColumns,
                !self.layout.is_default(),
                l10n,
                &forward,
                ProcessesSettingsMsg::ResetColumns,
            ),
            palette,
            &forward,
        );
        let sections = expander(
            self.heading(
                Group::Sections,
                l10n.processes_settings_sections(),
                l10n.processes_settings_sections_description(),
            ),
            sections.collect(),
            reset_row(
                l10n.processes_settings_sections_reset(),
                ProcessesSettingsMark::ResetSections,
                self.sections != SectionOrder::default(),
                l10n,
                &forward,
                ProcessesSettingsMsg::ResetSections,
            ),
            palette,
            &forward,
        );

        settings_column((
            crumbs(l10n, palette, self.back_hovered, &forward, back),
            StackPanel::new()
                .margin(Thickness::new(0.0, space::Section, 0.0, 0.0))
                .spacing(setting::CardSpacing)
                .children((columns, sections, self.memory_card(l10n, palette, &forward))),
        ))
    }
}

fn crumbs(
    l10n: &L10n,
    palette: Palette,
    hovered: bool,
    forward: &Callback<ProcessesSettingsMsg>,
    back: Callback<()>,
) -> View {
    let forward = forward.clone();
    breadcrumb(
        ProcessesSettingsMark::Back,
        Breadcrumb {
            parent: l10n.processes_title(),
            current: l10n.processes_settings_title(),
            hovered,
        },
        palette,
        Callback::new(move |hovered| {
            forward.call(ProcessesSettingsMsg::BackHovered(hovered));
        }),
        back,
    )
}

struct Heading {
    group: Group,
    title: String,
    description: String,
    open: bool,
}

fn expander(
    heading: Heading,
    rows: Vec<(String, View)>,
    reset: View,
    palette: Palette,
    forward: &Callback<ProcessesSettingsMsg>,
) -> View {
    let Heading {
        group,
        title,
        description,
        open,
    } = heading;
    let header = Border::new()
        .margin(Thickness::xy(0.0, setting::ExpanderHeaderInset))
        .content(card_words(title, Some(description), palette));
    let expanded = forward.clone();
    setting_expander()
        .mark(group.mark())
        .is_expanded(open)
        .on_is_expanded_changed(move |open: bool| {
            expanded.call(ProcessesSettingsMsg::Expand(group, open))
        })
        .header(header)
        .content(expander_rows(
            0.0,
            rows.into_iter()
                .chain([("reset".to_string(), reset)])
                .collect(),
        ))
        .into()
}

fn shown_switch(
    column: ProcessColumn,
    visible: bool,
    l10n: &L10n,
    palette: Palette,
    forward: &Callback<ProcessesSettingsMsg>,
) -> View {
    let state = if visible {
        l10n.processes_settings_shown_on()
    } else {
        l10n.processes_settings_shown_off()
    };
    let shown = forward.clone();
    setting_switch(
        ProcessesSettingsMark::Shown,
        visible,
        column != ProcessColumn::Name,
        state,
        palette,
        move |on: bool| shown.call(ProcessesSettingsMsg::ShowColumn(column, on)),
    )
}

fn reset_row(
    label: String,
    mark: ProcessesSettingsMark,
    enabled: bool,
    l10n: &L10n,
    forward: &Callback<ProcessesSettingsMsg>,
    message: ProcessesSettingsMsg,
) -> View {
    let forward = forward.clone();
    let button = action_button(
        mark,
        l10n.processes_settings_reset(),
        None,
        enabled,
        move || {
            forward.call(message);
        },
    );
    Grid::new()
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .children((
            text(label)
                .vertical_alignment(VerticalAlignment::Center)
                .grid_column(0),
            Grid::new()
                .grid_column(1)
                .vertical_alignment(VerticalAlignment::Center)
                .children((button,)),
        ))
        .into()
}

fn move_buttons(
    can: impl Fn(Step) -> bool,
    forward: &Callback<ProcessesSettingsMsg>,
    message: impl Fn(Step) -> ProcessesSettingsMsg + Clone + 'static,
) -> View {
    let button = |mark: ProcessesSettingsMark, step: Step, icon: View| {
        let forward = forward.clone();
        let message = message.clone();
        Button::new()
            .mark(mark)
            .style(ButtonStyle::Subtle)
            .is_enabled(can(step))
            .on_click(move || {
                forward.call(message(step));
            })
            .content(icon)
    };
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Compact)
        .children((
            button(
                ProcessesSettingsMark::Up,
                Step::Up,
                icon!(chevron_up_regular).size(size::Icon).build_element(),
            ),
            button(
                ProcessesSettingsMark::Down,
                Step::Down,
                icon!(chevron_down_regular).size(size::Icon).build_element(),
            ),
        ))
        .into()
}

fn row(mark: impl Mark, label: String, (moves, control): (View, Option<View>)) -> (String, View) {
    let key = mark.name().to_string();
    let mut cells: Vec<View> = vec![
        text(label)
            .vertical_alignment(VerticalAlignment::Center)
            .grid_column(0)
            .into(),
        Grid::new()
            .grid_column(1)
            .vertical_alignment(VerticalAlignment::Center)
            .children((moves,))
            .into(),
    ];
    cells.extend(control.map(|control| {
        Grid::new()
            .grid_column(2)
            .vertical_alignment(VerticalAlignment::Center)
            .children((control,))
            .into()
    }));
    let line = Border::new().mark(mark).content(
        Grid::new()
            .columns([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
            .column_spacing(space::Card)
            .children(cells),
    );
    (key, line.into())
}
