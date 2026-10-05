use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityState, DeleteGroup, DropRule, Group, Hue, MoveGroup, Pick, RecolorGroup, RenameGroup,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    keyed, Border, Button, ButtonStyle, Callback, Grid, GridLength, KeyedView, Orientation, StackPanel, TextBox,
    ThemeBrush, Thickness, TooltipExt, VerticalAlignment, View,
};

use super::components::groups::{group_label, swatch};
use super::components::picks::named;
use super::marks::ActivityGroupsMark;
use crate::l10n::L10n;
use crate::theme::{radius, setting, size, space, Palette};
use crate::widgets::breadcrumb::{breadcrumb, Breadcrumb};
use crate::widgets::button::command_button;
use crate::widgets::card::card;
use crate::widgets::settings_column::settings_column;
use crate::widgets::text::{body_strong, caption};

struct NameBox;

#[expect(non_upper_case_globals)]
impl NameBox {
    const Width: f64 = 260.0;
}

#[derive(Clone)]
pub enum ActivityGroupsMsg {
    BackHovered(bool),
}

#[derive(Default)]
pub struct ActivityGroupsPage {
    back_hovered: bool,
}

fn next_hue(hue: Hue) -> Hue {
    let at = Hue::ALL.iter().position(|each| *each == hue).unwrap_or(0);
    Hue::ALL[(at + 1) % Hue::ALL.len()]
}

fn rule_label(rule: &Pick, l10n: &L10n) -> String {
    match rule {
        Pick::Exe(_) => l10n.activity_groups_rule_exe(named(rule)),
        Pick::Folder(_) => l10n.activity_groups_rule_folder(named(rule)),
        Pick::Under(_) => l10n.activity_groups_rule_under(named(rule)),
    }
}

fn tool(mark: ActivityGroupsMark, glyph: View, hint: String, on_click: impl Fn() + 'static) -> View {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .on_click(on_click)
        .content(glyph)
        .tooltip(hint)
}

struct Place {
    first: bool,
    last: bool,
}

fn title(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    if group.is_built_in() {
        return StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(space::Control)
            .vertical_alignment(VerticalAlignment::Center)
            .children((
                body_strong(group_label(group, l10n)),
                caption(l10n.activity_groups_built_in())
                    .mark(ActivityGroupsMark::BuiltIn)
                    .foreground(palette.secondary_text)
                    .vertical_alignment(VerticalAlignment::Center),
            ))
            .into();
    }
    let (dispatch, id) = (dispatch.clone(), group.id.clone());
    TextBox::new(&group.name)
        .mark(ActivityGroupsMark::Name)
        .width(NameBox::Width)
        .placeholder_text(l10n.activity_groups_name())
        .vertical_alignment(VerticalAlignment::Center)
        .on_text_changed(move |name: Rc<str>| {
            dispatch.emit(RenameGroup {
                group: id.clone(),
                name: name.to_string(),
            })
        })
        .into()
}

fn tools(group: &Group, place: &Place, dispatch: &Dispatch, l10n: &L10n) -> View {
    let mut buttons: Vec<KeyedView> = Vec::new();
    let moved = |up: bool| {
        let (dispatch, id) = (dispatch.clone(), group.id.clone());
        move || dispatch.emit(MoveGroup { group: id.clone(), up })
    };
    if !place.first {
        buttons.push(keyed(
            "up",
            tool(
                ActivityGroupsMark::Up,
                icon!(chevron_up_regular).size(size::Icon).build_element(),
                l10n.activity_groups_up(),
                moved(true),
            ),
        ));
    }
    if !place.last {
        buttons.push(keyed(
            "down",
            tool(
                ActivityGroupsMark::Down,
                icon!(chevron_down_regular).size(size::Icon).build_element(),
                l10n.activity_groups_down(),
                moved(false),
            ),
        ));
    }
    if !group.is_built_in() {
        let (dispatch, id) = (dispatch.clone(), group.id.clone());
        buttons.push(keyed(
            "delete",
            tool(
                ActivityGroupsMark::Delete,
                icon!(delete).size(size::Icon).build_element(),
                l10n.activity_groups_delete(),
                move || dispatch.emit(DeleteGroup(id.clone())),
            ),
        ));
    }
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .vertical_alignment(VerticalAlignment::Center)
        .keyed_children(buttons)
        .into()
}

fn rules(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    if group.rules.is_empty() {
        return caption(l10n.activity_groups_empty()).foreground(palette.secondary_text).into();
    }
    let chips: Vec<KeyedView> = group
        .rules
        .iter()
        .map(|rule| {
            let (dispatch, id, dropped) = (dispatch.clone(), group.id.clone(), rule.clone());
            keyed(
                rule.id(),
                command_button(
                    ActivityGroupsMark::Rule,
                    rule_label(rule, l10n),
                    Some(icon!(dismiss).size(size::Icon).build_element()),
                    true,
                    move || {
                        dispatch.emit(DropRule {
                            group: id.clone(),
                            rule: dropped.clone(),
                        })
                    },
                ),
            )
        })
        .collect();
    StackPanel::new().keyed_children(chips).into()
}

fn group_card(group: &Group, place: Place, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let (recolor, id, hue) = (dispatch.clone(), group.id.clone(), group.hue);
    let color = Button::new()
        .mark(ActivityGroupsMark::Recolor)
        .style(ButtonStyle::Subtle)
        .vertical_alignment(VerticalAlignment::Center)
        .on_click(move || {
            recolor.emit(RecolorGroup {
                group: id.clone(),
                hue: next_hue(hue),
            })
        })
        .content(swatch(palette.hue(group.hue), true))
        .tooltip(l10n.activity_groups_recolor());
    let head = Grid::new()
        .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
        .column_spacing(space::Control)
        .children((
            Border::new().grid_column(0).content(color),
            Border::new().grid_column(1).vertical_alignment(VerticalAlignment::Center).content(title(
                group, dispatch, l10n, palette,
            )),
            Border::new().grid_column(2).content(tools(group, &place, dispatch, l10n)),
        ));
    card()
        .mark(ActivityGroupsMark::Group)
        .corner_radius(radius::Control)
        .border_thickness(Thickness::uniform(space::Hairline))
        .border_brush(ThemeBrush::CardStroke)
        .padding(Thickness::uniform(space::Card))
        .content(StackPanel::new().spacing(space::Control).children((
            head,
            Border::new()
                .margin(Thickness::new(setting::CaptionInset, 0.0, 0.0, 0.0))
                .content(rules(group, dispatch, l10n, palette)),
        )))
        .into()
}

impl ActivityGroupsPage {
    pub fn update(&mut self, message: ActivityGroupsMsg) {
        match message {
            ActivityGroupsMsg::BackHovered(hovered) => self.back_hovered = hovered,
        }
    }

    pub fn view(
        &self,
        state: &ActivityState,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<ActivityGroupsMsg>,
        back: Callback<()>,
    ) -> View {
        let crumbs = breadcrumb(
            ActivityGroupsMark::Back,
            Breadcrumb {
                parent: l10n.activity_title(),
                current: l10n.activity_groups_title(),
                hovered: self.back_hovered,
            },
            palette,
            Callback::new(move |hovered| forward.call(ActivityGroupsMsg::BackHovered(hovered))),
            back,
        );
        let last = state.groups.len().saturating_sub(1);
        let cards: Vec<KeyedView> = state
            .groups
            .iter()
            .enumerate()
            .map(|(at, group)| {
                let place = Place {
                    first: at == 0,
                    last: at == last,
                };
                keyed(group.id.clone(), group_card(group, place, dispatch, l10n, palette))
            })
            .collect();
        settings_column((
            crumbs,
            caption(l10n.activity_groups_order())
                .foreground(palette.secondary_text)
                .margin(Thickness::new(setting::CaptionInset, space::Section, 0.0, space::Control)),
            StackPanel::new().spacing(setting::CardSpacing).keyed_children(cards),
        ))
    }
}
