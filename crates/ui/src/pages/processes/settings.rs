use amethystate::ReactiveMap;
use app_contracts::features::processes::ProcessColumn;
use guicons::icon;
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Button, ButtonStyle, Callback, ChildrenControl, ContentControl, Grid, GridChildExt, GridLength,
    KeyedView, LayoutControl, Orientation, ScrollViewer, StackPanel, ThemeBrush, Thickness, ToggleSwitch,
    VerticalAlignment, View,
};

use super::components::column_layout::ColumnLayout;
use super::components::columns::{column_label, section_label};
use super::components::grouping::{SectionId, SectionOrder};
use super::components::Step;
use super::marks::ProcessesSettingsMark;
use super::page::ProcessesSettingsMaps;
use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::card::card;
use crate::widgets::page::action_button;
use crate::widgets::text::{body_strong, caption, subtitle, text};

struct Layout;

#[expect(non_upper_case_globals)]
impl Layout {
    const MaxWidth: f64 = 1064.0;
    const RowSpacing: f64 = 4.0;
    const RowMinHeight: f64 = 48.0;
    const Border: f64 = 1.0;

    fn section_header() -> Thickness {
        Thickness::new(1.0, 30.0, 0.0, 6.0)
    }
}

pub enum ProcessesSettingsMsg {
    ShowColumn(ProcessColumn, bool),
    MoveColumn(ProcessColumn, Step),
    MoveSection(SectionId, Step),
    ResetSections,
}

pub struct ProcessesSettingsPage {
    layout: ColumnLayout,
    sections: SectionOrder,
    section_ranks: Option<ReactiveMap<String, u32>>,
}

impl Default for ProcessesSettingsPage {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ProcessesSettingsPage {
    pub fn new(settings: Option<ProcessesSettingsMaps>) -> Self {
        let (columns, column_order, section_ranks) = match settings {
            Some(maps) => (Some(maps.columns), Some(maps.column_order), Some(maps.section_order)),
            None => (None, None, None),
        };
        Self {
            layout: ColumnLayout::new(columns, column_order),
            sections: SectionOrder::kept(section_ranks.as_ref()),
            section_ranks,
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
            ProcessesSettingsMsg::ResetSections => {
                self.sections = SectionOrder::default();
                if let Some(ranks) = &self.section_ranks {
                    SectionOrder::forget(ranks);
                }
            }
        }
    }

    pub fn view(
        &self,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ProcessesSettingsMsg>,
        back: Callback<()>,
    ) -> View {
        let columns = self.layout.placed().into_iter().map(|column| {
            let shown = forward.clone();
            let switch = ToggleSwitch::new()
                .mark(ProcessesSettingsMark::Shown)
                .is_on(self.layout.visible(column))
                .is_enabled(column != ProcessColumn::Name)
                .on_toggled(move |on: bool| {
                    let _ = shown.call(ProcessesSettingsMsg::ShowColumn(column, on));
                });
            let moves = move_buttons(
                |step| self.layout.can_shift(column, step),
                &forward,
                move |step| ProcessesSettingsMsg::MoveColumn(column, step),
            );
            row(column, column_label(l10n, column), (moves, switch.into()))
        });

        let sections = self.sections.ids().iter().map(|&section| {
            let moves = move_buttons(
                |step| self.sections.can_shift(section, step),
                &forward,
                move |step| ProcessesSettingsMsg::MoveSection(section, step),
            );
            row(section, section_label(l10n, section), (moves, View::empty()))
        });

        let reset = forward.clone();
        let reset = action_button(
            ProcessesSettingsMark::ResetSections,
            l10n.processes_settings_reset_sections(),
            Some(icon!(restore).size(size::Icon).build_element()),
            self.sections != SectionOrder::default(),
            move || {
                let _ = reset.call(ProcessesSettingsMsg::ResetSections);
            },
        );

        ScrollViewer::new()
            .content(
                StackPanel::new()
                    .max_width(Layout::MaxWidth)
                    .margin(Thickness::new(space::Page, space::Section, space::Page, space::Page))
                    .children((
                        breadcrumb(l10n, palette, back),
                        section(
                            l10n.processes_settings_columns(),
                            l10n.processes_settings_columns_description(),
                            View::empty(),
                            columns.collect::<Vec<_>>(),
                            palette,
                        ),
                        section(
                            l10n.processes_settings_sections(),
                            l10n.processes_settings_sections_description(),
                            reset,
                            sections.collect::<Vec<_>>(),
                            palette,
                        ),
                    )),
            )
            .into()
    }
}

fn breadcrumb(l10n: &L10n, palette: Palette, back: Callback<()>) -> View {
    let parent = Button::new()
        .mark(ProcessesSettingsMark::Back)
        .style(ButtonStyle::Subtle)
        .on_click(move || {
            let _ = back.call(());
        })
        .content(subtitle(l10n.processes_title()).foreground(palette.secondary_text));
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Control)
        .children((
            parent,
            Grid::new()
                .vertical_alignment(VerticalAlignment::Center)
                .children((icon!(chevron_right_regular).size(size::Icon).build_element(),)),
            subtitle(l10n.processes_settings_title()).vertical_alignment(VerticalAlignment::Center),
        ))
        .into()
}

fn section(title: String, description: String, action: View, rows: Vec<KeyedView>, palette: Palette) -> View {
    let heading = Grid::new()
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .margin(Layout::section_header())
        .children((
            StackPanel::new().grid_column(0).children((
                body_strong(title),
                caption(description).foreground(palette.secondary_text),
            )),
            Grid::new()
                .grid_column(1)
                .vertical_alignment(VerticalAlignment::Bottom)
                .children((action,)),
        ));
    StackPanel::new()
        .children((
            heading,
            StackPanel::new()
                .spacing(Layout::RowSpacing)
                .children((View::keyed_fragment(rows),)),
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
                let _ = forward.call(message(step));
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

fn row(mark: impl Mark, label: String, (moves, control): (View, View)) -> KeyedView {
    let key = mark.name();
    let row = card()
        .mark(mark)
        .corner_radius(radius::Control)
        .border_thickness(Layout::Border)
        .border_brush(ThemeBrush::CardStroke)
        .min_height(Layout::RowMinHeight)
        .padding(Thickness::xy(space::Card, space::Compact))
        .content(
            Grid::new()
                .columns([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
                .column_spacing(space::Card)
                .children((
                    text(label)
                        .vertical_alignment(VerticalAlignment::Center)
                        .grid_column(0),
                    Grid::new()
                        .grid_column(1)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((moves,)),
                    Grid::new()
                        .grid_column(2)
                        .vertical_alignment(VerticalAlignment::Center)
                        .children((control,)),
                )),
        );
    KeyedView::new(key, row)
}
