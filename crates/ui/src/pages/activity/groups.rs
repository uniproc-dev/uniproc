use std::collections::HashSet;
use std::rc::Rc;

use app_contracts::features::activity::{
    ActivityState, DeleteGroup, DropRule, Group, Hue, MoveGroup, Pick, RecolorGroup, RenameGroup, ShowGroup, Unhide,
};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    keyed, Border, Button, ButtonStyle, Callback, CornerRadius, Grid, GridLength, KeyedView, Orientation, StackPanel,
    TextBox, ThemeBrush, Thickness, TooltipExt, VerticalAlignment, View,
};

use super::components::groups::{group_label, swatch, Swatch};
use super::components::picks::named;
use super::marks::ActivityGroupsMark;
use crate::l10n::L10n;
use crate::theme::{setting, size, space, Palette};
use crate::widgets::breadcrumb::{breadcrumb, Breadcrumb};
use crate::widgets::setting_card::{card_rows, expander_rows, setting_expander, setting_switch};
use crate::widgets::settings_column::{settings_column, settings_section};
use crate::widgets::text::{caption, text};

struct Layout;

#[expect(non_upper_case_globals)]
impl Layout {
    const NameWidth: f64 = 220.0;
    const Ring: f64 = 18.0;
    const RingWidth: f64 = 2.0;
    const RuleIcon: f64 = 20.0;
}

#[derive(Clone)]
pub enum ActivityGroupsMsg {
    BackHovered(bool),
    Expand(String, bool),
}

#[derive(Default)]
pub struct ActivityGroupsPage {
    back_hovered: bool,
    expanded: HashSet<String>,
}

fn rule_label(rule: &Pick, l10n: &L10n) -> View {
    let (glyph, kind) = match rule {
        Pick::Exe(_) => (icon!(rule_exe), l10n.activity_groups_kind_exe()),
        Pick::Folder(_) => (icon!(rule_folder), l10n.activity_groups_kind_folder()),
        Pick::Under(_) => (icon!(rule_under), l10n.activity_groups_kind_under()),
    };
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Control)
        .children((
            Border::new()
                .vertical_alignment(VerticalAlignment::Center)
                .content(glyph.size(Layout::RuleIcon).build_element()),
            text(named(rule)).vertical_alignment(VerticalAlignment::Center),
        ))
        .tooltip(kind)
}

fn tool(mark: ActivityGroupsMark, glyph: View, hint: String, on_click: impl Fn() + 'static) -> View {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .on_click(on_click)
        .content(glyph)
        .tooltip(hint)
}

fn line(main: View, trailing: View) -> View {
    Grid::new()
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .children((
            Border::new().grid_column(0).vertical_alignment(VerticalAlignment::Center).content(main),
            Border::new().grid_column(1).vertical_alignment(VerticalAlignment::Center).content(trailing),
        ))
        .into()
}

fn shown_switch(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let (dispatch, id) = (dispatch.clone(), group.id.clone());
    let state = if group.shown {
        l10n.activity_groups_shown_on()
    } else {
        l10n.activity_groups_shown_off()
    };
    setting_switch(ActivityGroupsMark::Shown, group.shown, true, state, palette, move |shown: bool| {
        dispatch.emit(ShowGroup {
            group: id.clone(),
            shown,
        })
    })
}

fn header(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let count = group.rules.len() as i64;
    let summary = if group.is_built_in() {
        caption(l10n.activity_groups_built_in_rules(count)).mark(ActivityGroupsMark::BuiltIn)
    } else {
        caption(l10n.activity_groups_rules(count))
    };
    Grid::new()
        .columns([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
        .column_spacing(space::Header)
        .margin(Thickness::xy(0.0, setting::ExpanderHeaderInset))
        .children((
            Border::new()
                .grid_column(0)
                .vertical_alignment(VerticalAlignment::Center)
                .content(swatch(palette.hue(group.hue), true)),
            Border::new()
                .grid_column(1)
                .vertical_alignment(VerticalAlignment::Center)
                .content(text(group_label(group, l10n))),
            Border::new()
                .grid_column(2)
                .vertical_alignment(VerticalAlignment::Center)
                .content(summary.foreground(palette.secondary_text)),
            Border::new()
                .grid_column(3)
                .vertical_alignment(VerticalAlignment::Center)
                .content(shown_switch(group, dispatch, l10n, palette)),
        ))
        .into()
}

fn rules(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> Vec<(String, View)> {
    if group.rules.is_empty() {
        return vec![(
            "empty".to_string(),
            caption(l10n.activity_groups_empty()).foreground(palette.secondary_text).into(),
        )];
    }
    group
        .rules
        .iter()
        .map(|rule| {
            let (dispatch, id, dropped) = (dispatch.clone(), group.id.clone(), rule.clone());
            let drop = tool(
                ActivityGroupsMark::Rule,
                icon!(dismiss).size(size::Icon).build_element(),
                l10n.activity_groups_drop_rule(),
                move || {
                    dispatch.emit(DropRule {
                        group: id.clone(),
                        rule: dropped.clone(),
                    })
                },
            );
            (rule.id(), line(rule_label(rule, l10n), drop))
        })
        .collect()
}

fn colours(group: &Group, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let picks: Vec<KeyedView> = Hue::ALL
        .into_iter()
        .map(|hue| {
            let (dispatch, id) = (dispatch.clone(), group.id.clone());
            let ring = Border::new()
                .width(Layout::Ring)
                .height(Layout::Ring)
                .corner_radius(CornerRadius::uniform(Layout::Ring / 2.0))
                .border_thickness(Thickness::uniform(Layout::RingWidth))
                .content(swatch(palette.hue(hue), true));
            let ring = if group.hue == hue {
                ring.border_brush(ThemeBrush::PrimaryText)
            } else {
                ring
            };
            keyed(
                hue.id(),
                Button::new()
                    .mark(ActivityGroupsMark::Colour)
                    .style(ButtonStyle::Subtle)
                    .on_click(move || dispatch.emit(RecolorGroup { group: id.clone(), hue }))
                    .content(ring)
                    .tooltip(l10n.activity_groups_colour()),
            )
        })
        .collect();
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .vertical_alignment(VerticalAlignment::Center)
        .keyed_children(picks)
        .into()
}

struct Place {
    first: bool,
    last: bool,
}

fn edit(group: &Group, place: Place, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let mut parts: Vec<KeyedView> = Vec::new();
    if !group.is_built_in() {
        let (renamed, id) = (dispatch.clone(), group.id.clone());
        parts.push(keyed(
            "name",
            TextBox::new(&group.name)
                .mark(ActivityGroupsMark::Name)
                .width(Layout::NameWidth)
                .placeholder_text(l10n.activity_groups_name())
                .vertical_alignment(VerticalAlignment::Center)
                .on_text_changed(move |name: Rc<str>| {
                    renamed.emit(RenameGroup {
                        group: id.clone(),
                        name: name.to_string(),
                    })
                }),
        ));
    }
    parts.push(keyed("colours", colours(group, dispatch, l10n, palette)));

    let mut tools: Vec<KeyedView> = Vec::new();
    let moved = |up: bool| {
        let (dispatch, id) = (dispatch.clone(), group.id.clone());
        move || dispatch.emit(MoveGroup { group: id.clone(), up })
    };
    if !place.first {
        tools.push(keyed(
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
        tools.push(keyed(
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
        tools.push(keyed(
            "delete",
            tool(
                ActivityGroupsMark::Delete,
                icon!(delete).size(size::Icon).build_element(),
                l10n.activity_groups_delete(),
                move || dispatch.emit(DeleteGroup(id.clone())),
            ),
        ));
    }
    let main = StackPanel::new().orientation(Orientation::Horizontal).spacing(space::Header).keyed_children(parts);
    let tools = StackPanel::new().orientation(Orientation::Horizontal).keyed_children(tools);
    line(main.into(), tools.into())
}

fn hidden_section(state: &ActivityState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let hidden = &state.filter.hidden;
    let body: View = if hidden.is_empty() {
        caption(l10n.activity_groups_nothing_hidden())
            .mark(ActivityGroupsMark::NothingHidden)
            .foreground(palette.secondary_text)
            .margin(Thickness::new(setting::CaptionInset, 0.0, 0.0, 0.0))
            .into()
    } else {
        let rows: Vec<(String, View)> = hidden
            .iter()
            .map(|pick| {
                let (dispatch, shown) = (dispatch.clone(), pick.clone());
                let again = tool(
                    ActivityGroupsMark::ShowAgain,
                    icon!(dismiss).size(size::Icon).build_element(),
                    l10n.activity_groups_show_again(),
                    move || dispatch.emit(Unhide(shown.clone())),
                );
                let row = Border::new().mark(ActivityGroupsMark::Hidden).content(line(rule_label(pick, l10n), again));
                (pick.id(), row.into())
            })
            .collect();
        card_rows(rows)
    };
    settings_section(l10n.activity_groups_hidden(), (body,))
}

impl ActivityGroupsPage {
    pub fn update(&mut self, message: ActivityGroupsMsg) {
        match message {
            ActivityGroupsMsg::BackHovered(hovered) => self.back_hovered = hovered,
            ActivityGroupsMsg::Expand(group, true) => {
                self.expanded.insert(group);
            }
            ActivityGroupsMsg::Expand(group, false) => {
                self.expanded.remove(&group);
            }
        }
    }

    fn group(
        &self,
        group: &Group,
        place: Place,
        dispatch: &Dispatch,
        l10n: &L10n,
        palette: Palette,
        forward: &Callback<ActivityGroupsMsg>,
    ) -> View {
        let mut content = rules(group, dispatch, l10n, palette);
        content.push(("edit".to_string(), edit(group, place, dispatch, l10n, palette)));
        let (expanded, id) = (forward.clone(), group.id.clone());
        setting_expander()
            .mark(ActivityGroupsMark::Group)
            .is_expanded(self.expanded.contains(&group.id))
            .on_is_expanded_changed(move |open: bool| expanded.call(ActivityGroupsMsg::Expand(id.clone(), open)))
            .header(header(group, dispatch, l10n, palette))
            .content(expander_rows(Swatch::Size + space::Header, content).mark(ActivityGroupsMark::Rows))
            .into()
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
        let hover = forward.clone();
        let crumbs = breadcrumb(
            ActivityGroupsMark::Back,
            Breadcrumb {
                parent: l10n.activity_title(),
                current: l10n.activity_groups_title(),
                hovered: self.back_hovered,
            },
            palette,
            Callback::new(move |hovered| hover.call(ActivityGroupsMsg::BackHovered(hovered))),
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
                keyed(group.id.clone(), self.group(group, place, dispatch, l10n, palette, &forward))
            })
            .collect();
        settings_column((
            crumbs,
            caption(l10n.activity_groups_order())
                .foreground(palette.secondary_text)
                .margin(Thickness::new(setting::CaptionInset, space::Section, 0.0, space::Control)),
            StackPanel::new().spacing(setting::CardSpacing).keyed_children(cards),
            hidden_section(state, dispatch, l10n, palette),
        ))
    }
}
