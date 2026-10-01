use app_contracts::features::agents::AgentConnectionState;
use windows_reactor::{
    Border, ChildrenControl, ContentControl, HorizontalAlignment, LayoutControl, Orientation,
    ProgressRing, StackPanel, Thickness, VerticalAlignment, View,
};

use crate::l10n::L10n;
use crate::theme::{radius, size, space, Palette};
use crate::widgets::text::text;

pub(crate) fn disconnected_overlay(l10n: &L10n, palette: Palette, agent: AgentConnectionState) -> View {
    let status = match agent {
        AgentConnectionState::GaveUp => l10n.processes_agent_gave_up(),
        AgentConnectionState::Outdated => l10n.processes_agent_outdated(),
        _ => l10n.processes_connecting(),
    };
    Border::new()
        .background(palette.layer_fill)
        .corner_radius(radius::Overlay)
        .padding(Thickness::xy(space::Card, space::Header))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Top)
        .margin(Thickness::xy(0.0, space::Header))
        .content(
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Header)
                .children((
                    ProgressRing::new()
                        .is_indeterminate(true)
                        .width(size::NavIcon)
                        .height(size::NavIcon),
                    text(status),
                )),
        )
}
