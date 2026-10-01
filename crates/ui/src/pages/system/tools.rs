use app_contracts::features::system::{SystemState, SystemTool, ToolGroup};
use guinea::prelude::Dispatch;
use guinea::Mark;
use windows_reactor::{Callback, KeyedView, LayoutControl, Thickness, View};

use super::components::tool_card::{group_title, tool_card, CardPlace};
use super::marks::{SystemMark, ToolMark};
use crate::l10n::L10n;
use crate::theme::{setting, space, Palette};
use crate::widgets::breadcrumb::{breadcrumb, Breadcrumb};
use crate::widgets::page::{settings_column, settings_section};
use crate::widgets::text::caption;

pub struct ToolsProps<'a> {
    pub state: &'a SystemState,
    pub dispatch: &'a Dispatch,
    pub l10n: &'a L10n,
    pub palette: Palette,
    pub back_hovered: bool,
    pub on_back_hover: Callback<bool>,
    pub back: Callback<()>,
}

fn group_section(props: &ToolsProps<'_>, group: ToolGroup) -> View {
    let ToolsProps {
        state,
        dispatch,
        l10n,
        palette,
        ..
    } = *props;
    let hint = match group {
        ToolGroup::Sysinternals => caption(l10n.system_sysinternals_hint())
            .foreground(palette.secondary_text)
            .margin(Thickness::new(setting::CaptionInset, 0.0, 0.0, space::Control))
            .into(),
        _ => View::empty(),
    };
    let cards = SystemTool::in_group(group).map(|tool| {
        KeyedView::new(
            ToolMark(tool).name(),
            tool_card(state, dispatch, l10n, palette, tool, CardPlace::Catalog),
        )
    });
    settings_section(group_title(l10n, group), (hint, View::keyed_fragment(cards)))
}

pub fn tools_view(props: ToolsProps<'_>) -> View {
    let crumbs = breadcrumb(
        SystemMark::Back,
        Breadcrumb {
            parent: props.l10n.system_title(),
            current: props.l10n.system_tools(),
            hovered: props.back_hovered,
        },
        props.palette,
        props.on_back_hover.clone(),
        props.back.clone(),
    );
    let [windows, settings, sysinternals] = ToolGroup::ALL.map(|group| group_section(&props, group));
    settings_column((crumbs, windows, settings, sysinternals))
}
