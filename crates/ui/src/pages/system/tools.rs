use app_contracts::features::system::{SystemState, SystemTool, ToolGroup};
use guinea::prelude::Dispatch;
use windows_reactor::{Callback, Thickness, View};

use super::components::tool_card::{CardPlace, group_title, tool_card};
use super::marks::SystemMark;
use crate::l10n::L10n;
use crate::theme::{Palette, setting, space};
use crate::widgets::breadcrumb::{Breadcrumb, breadcrumb};
use crate::widgets::settings_column::{settings_column, settings_section};
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
    let mut content: Vec<View> = Vec::new();
    if group == ToolGroup::Sysinternals {
        content.push(
            caption(l10n.system_sysinternals_hint())
                .foreground(palette.secondary_text)
                .margin(Thickness::new(
                    setting::CaptionInset,
                    0.0,
                    0.0,
                    space::Control,
                ))
                .into(),
        );
    }
    content.extend(
        SystemTool::in_group(group)
            .map(|tool| tool_card(state, dispatch, l10n, palette, tool, CardPlace::Catalog)),
    );
    settings_section(group_title(l10n, group), content)
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
    let [windows, settings, sysinternals] =
        ToolGroup::ALL.map(|group| group_section(&props, group));
    settings_column((crumbs, windows, settings, sysinternals))
}
