use app_contracts::features::activity::{Filter, Pick};
use guicons::icon;
use windows_reactor::{keyed, Callback, KeyedView, Orientation, StackPanel, Thickness, View};

use super::super::marks::ActivityMark;
use crate::l10n::L10n;
use crate::theme::{size, space};
use crate::widgets::button::command_button;
use crate::widgets::popup_menu::MenuLine;

pub fn named(pick: &Pick) -> String {
    match pick {
        Pick::Exe(path) => path.rsplit(['\\', '/']).next().unwrap_or(path).to_string(),
        Pick::Folder(folder) => folder.to_string(),
        Pick::Launcher(name) => name.to_string(),
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
}

pub fn pick_lines(picks: &[Pick], l10n: &L10n) -> Vec<MenuLine<ActivityMark, PickCommand>> {
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
            Pick::Launcher(name) => lines.push(MenuLine::entry(
                ActivityMark::HideLauncher,
                hide(),
                l10n.activity_pick_hide_launcher(name.to_string()),
                hidden,
            )),
        }
    }
    lines
}

pub fn picked(filter: &Filter, l10n: &L10n, on_all: Callback<()>, on_unhide: Callback<Pick>) -> Option<View> {
    let mut chips: Vec<KeyedView> = Vec::new();
    if let Some(only) = &filter.only {
        chips.push(keyed(
            "only",
            command_button(
                ActivityMark::Picked,
                l10n.activity_picked_only(named(only)),
                Some(icon!(dismiss).size(size::Icon).build_element()),
                true,
                move || on_all.call(()),
            ),
        ));
    }
    for pick in &filter.hidden {
        let (on_unhide, shown) = (on_unhide.clone(), pick.clone());
        chips.push(keyed(
            format!("{pick:?}"),
            command_button(
                ActivityMark::Picked,
                l10n.activity_picked_hidden(named(pick)),
                Some(icon!(dismiss).size(size::Icon).build_element()),
                true,
                move || on_unhide.call(shown.clone()),
            ),
        ));
    }
    (!chips.is_empty()).then(|| row(chips, Thickness::xy(space::Cell, 0.0)))
}
