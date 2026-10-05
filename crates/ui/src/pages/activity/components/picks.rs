use app_contracts::features::activity::{Filter, Group, Pick};
use guicons::icon;
use windows_reactor::{keyed, Callback, KeyedView, Orientation, StackPanel, Thickness, View};

use super::super::marks::ActivityMark;
use super::groups::{group_label, swatch};
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::button::command_button;
use crate::widgets::popup_menu::MenuLine;

pub fn named(pick: &Pick) -> String {
    match pick {
        Pick::Exe(name) | Pick::Under(name) => name.to_string(),
        Pick::Folder(folder) => folder.to_string(),
    }
}

fn row(children: Vec<KeyedView>, margin: Thickness) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Control)
        .margin(margin)
        .keyed_children(children)
        .into()
}

#[derive(Clone, PartialEq, Debug)]
pub enum PickCommand {
    Only(Pick),
    Hide(Pick),
    Put { group: String, rule: Pick },
    New(Pick),
}

fn group_lines(
    heading: String,
    rule: &Pick,
    groups: &[Group],
    l10n: &L10n,
    palette: Palette,
) -> Vec<MenuLine<ActivityMark, PickCommand>> {
    let mut lines = vec![MenuLine::Separator, MenuLine::Caption(heading)];
    for group in groups {
        lines.push(MenuLine::entry(
            ActivityMark::PutIn,
            swatch(palette.hue(group.hue), true),
            group_label(group, l10n),
            PickCommand::Put {
                group: group.id.clone(),
                rule: rule.clone(),
            },
        ));
    }
    lines.push(MenuLine::entry(
        ActivityMark::NewGroup,
        icon!(plus).size(size::Icon).build_element(),
        l10n.activity_pick_new_group(),
        PickCommand::New(rule.clone()),
    ));
    lines
}

pub fn pick_lines(
    picks: &[Pick],
    groups: &[Group],
    l10n: &L10n,
    palette: Palette,
) -> Vec<MenuLine<ActivityMark, PickCommand>> {
    let mut lines = hide_lines(picks, l10n);
    for pick in picks {
        match pick {
            Pick::Exe(name) => {
                lines.extend(group_lines(l10n.activity_pick_put_exe(name.to_string()), pick, groups, l10n, palette))
            }
            Pick::Under(name) => lines.extend(group_lines(
                l10n.activity_pick_put_under(name.to_string()),
                pick,
                groups,
                l10n,
                palette,
            )),
            Pick::Folder(_) => {}
        }
    }
    lines
}

fn hide_lines(picks: &[Pick], l10n: &L10n) -> Vec<MenuLine<ActivityMark, PickCommand>> {
    let hide = || icon!(eye_off).size(size::Icon).build_element();
    let mut lines = Vec::new();
    for pick in picks {
        let hidden = PickCommand::Hide(pick.clone());
        match pick {
            Pick::Exe(_) => {
                lines.push(MenuLine::entry(
                    ActivityMark::Only,
                    icon!(filter).size(size::Icon).build_element(),
                    l10n.activity_pick_only(),
                    PickCommand::Only(pick.clone()),
                ));
                lines.push(MenuLine::entry(ActivityMark::HideExe, hide(), l10n.activity_pick_hide_exe(), hidden));
            }
            Pick::Folder(_) => lines.push(MenuLine::entry(
                ActivityMark::HideFolder,
                hide(),
                l10n.activity_pick_hide_folder(),
                hidden,
            )),
            Pick::Under(name) => lines.push(MenuLine::entry(
                ActivityMark::HideLauncher,
                hide(),
                l10n.activity_pick_hide_launcher(name.to_string()),
                hidden,
            )),
        }
    }
    lines
}

pub fn picked(filter: &Filter, l10n: &L10n, on_all: Callback<()>) -> Option<View> {
    let only = filter.only.as_ref()?;
    let chip = command_button(
        ActivityMark::Picked,
        l10n.activity_picked_only(named(only)),
        Some(icon!(dismiss).size(size::Icon).build_element()),
        true,
        move || on_all.call(()),
    );
    Some(row(vec![keyed("only", chip)], Thickness::xy(space::Cell, 0.0)))
}
