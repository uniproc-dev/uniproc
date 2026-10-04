use app_contracts::features::activity::{Filter, Pick};
use guicons::icon;
use windows_reactor::{keyed, Callback, KeyedView, Orientation, StackPanel, Thickness, View};

use super::super::marks::ActivityMark;
use crate::l10n::L10n;
use crate::theme::{size, space};
use crate::widgets::button::command_button;

fn named(pick: &Pick) -> String {
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

pub fn pick_buttons(
    picks: &[Pick],
    l10n: &L10n,
    on_only: &Callback<Pick>,
    on_hide: &Callback<Pick>,
    indent: f64,
) -> View {
    let mut buttons: Vec<KeyedView> = Vec::new();
    for pick in picks {
        let hide = {
            let (on_hide, pick) = (on_hide.clone(), pick.clone());
            move || on_hide.call(pick.clone())
        };
        match pick {
            Pick::Exe(_) => {
                let (on_only, only) = (on_only.clone(), pick.clone());
                buttons.push(keyed(
                    "only",
                    command_button(ActivityMark::Only, l10n.activity_pick_only(), None, true, move || {
                        on_only.call(only.clone())
                    }),
                ));
                buttons.push(keyed(
                    "exe",
                    command_button(ActivityMark::HideExe, l10n.activity_pick_hide_exe(), None, true, hide),
                ));
            }
            Pick::Folder(_) => buttons.push(keyed(
                "folder",
                command_button(ActivityMark::HideFolder, l10n.activity_pick_hide_folder(), None, true, hide),
            )),
            Pick::Launcher(name) => buttons.push(keyed(
                "launcher",
                command_button(
                    ActivityMark::HideLauncher,
                    l10n.activity_pick_hide_launcher(name.to_string()),
                    None,
                    true,
                    hide,
                ),
            )),
        }
    }
    row(buttons, Thickness::new(indent + space::Control, 0.0, 0.0, space::Control))
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
