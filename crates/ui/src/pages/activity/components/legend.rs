use app_contracts::features::activity::{Filter, Group, Legend, ShowGroup, ShowOther};
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{keyed, Button, ButtonStyle, Color, KeyedView, Orientation, StackPanel, Thickness, View};

use super::super::marks::ActivityMark;
use super::groups::{group_label, swatch};
use crate::l10n::L10n;
use crate::theme::{opacity, space, Palette};
use crate::widgets::text::{caption, text};

fn chip(
    mark: ActivityMark,
    color: Color,
    label: String,
    count: usize,
    shown: bool,
    l10n: &L10n,
    palette: Palette,
    on_click: impl Fn() + 'static,
) -> Button {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .on_click(on_click)
        .content(
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .opacity(if shown { 1.0 } else { opacity::Disabled })
                .children((
                    swatch(color, shown),
                    text(label),
                    caption(l10n.activity_legend_count(count as i64)).foreground(palette.secondary_text),
                )),
        )
}

pub fn legend(groups: &[Group], filter: &Filter, counts: &Legend, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let mut chips: Vec<KeyedView> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let (dispatch, id, shown) = (dispatch.clone(), group.id.clone(), group.shown);
            keyed(
                group.id.clone(),
                chip(
                    ActivityMark::LegendGroup,
                    palette.hue(group.hue),
                    group_label(group, l10n),
                    counts.groups.get(index).copied().unwrap_or_default(),
                    shown,
                    l10n,
                    palette,
                    move || {
                        dispatch.emit(ShowGroup {
                            group: id.clone(),
                            shown: !shown,
                        })
                    },
                ),
            )
        })
        .collect();
    let (dispatch, other) = (dispatch.clone(), filter.other);
    chips.push(keyed(
        "other",
        chip(
            ActivityMark::LegendOther,
            palette.success,
            l10n.activity_legend_other(),
            counts.other,
            other,
            l10n,
            palette,
            move || dispatch.emit(ShowOther(!other)),
        ),
    ));
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Compact)
        .margin(Thickness::new(space::Cell, space::Card, space::Cell, 0.0))
        .keyed_children(chips)
        .into()
}
