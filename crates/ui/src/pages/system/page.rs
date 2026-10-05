use app_contracts::features::system::SystemState;
use guicons::icon;
use guinea::Mark;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{Callback, KeyedView, StackPanel, Thickness, View};

use super::components::tool_card::{CardPlace, tool_card};
use super::marks::{SystemMark, ToolMark};
use crate::l10n::L10n;
use crate::theme::{Palette, setting, space};
use crate::widgets::link_card::{LinkCard, Trailing, link_card};
use crate::widgets::settings_column::{settings_column, settings_section};
use crate::widgets::text::{caption, subtitle};

fn favourites(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let cards: Vec<KeyedView> = state
        .favourites()
        .map(|tool| {
            KeyedView::new(
                ToolMark(tool).name(),
                tool_card(state, dispatch, l10n, palette, tool, CardPlace::Favourites),
            )
        })
        .collect();
    let content: View = if cards.is_empty() {
        caption(l10n.system_favourites_empty())
            .mark(SystemMark::NoFavourites)
            .foreground(palette.secondary_text)
            .margin(Thickness::new(setting::CaptionInset, 0.0, 0.0, 0.0))
            .into()
    } else {
        StackPanel::new()
            .mark(SystemMark::Favourites)
            .spacing(setting::CardSpacing)
            .keyed_children(cards)
            .into()
    };
    settings_section(l10n.system_section_favourites(), (content,))
}

pub fn system_view(
    state: &SystemState,
    dispatch: &Dispatch,
    l10n: &L10n,
    palette: Palette,
    open_tools: Callback<()>,
) -> View {
    let tools = link_card(
        SystemMark::Tools,
        LinkCard {
            icon: Some(icon!(toolbox).size(setting::Icon).build_element()),
            title: l10n.system_tools(),
            description: Some(l10n.system_tools_description()),
            trailing: Trailing::Chevron,
            accessory: None,
        },
        palette,
        move || {
            open_tools.call(());
        },
    );
    settings_column((
        subtitle(l10n.system_title()),
        StackPanel::new()
            .margin(Thickness::new(0.0, space::Section, 0.0, 0.0))
            .children((tools,)),
        favourites(state, dispatch, l10n, palette),
    ))
}
