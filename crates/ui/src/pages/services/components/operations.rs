use std::rc::Rc;

use app_contracts::features::services::ServiceOperation;
use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, Grid, GridLength, HorizontalAlignment, KeyedView, ProgressRing,
    ScrollBarVisibility, ScrollViewer, StackPanel, TextTrimming, Thickness, TooltipExt, VerticalAlignment, View,
    keyed,
};

use super::super::marks::ServicesMark;
use crate::theme::{Palette, radius, size, space};
use crate::widgets::button::icon_button;
use crate::widgets::selection::SelectionMark;
use crate::widgets::text::text;

struct Panel;

#[expect(non_upper_case_globals)]
impl Panel {
    const Row: f64 = 44.0;
    const Indicator: f64 = 18.0;
}

#[derive(Clone, PartialEq)]
pub struct OperationsProps {
    pub operations: Rc<[ServiceOperation]>,
    pub palette: Palette,
    pub collapse: String,
    pub expand: String,
}

pub fn operations(
    props: &OperationsProps,
    visible: bool,
    collapsed: bool,
    toggle: Callback<()>,
) -> View {
    let palette = props.palette;
    let arrow = if collapsed {
        icon!(chevron_up_regular)
    } else {
        icon!(chevron_down_regular)
    };
    let toggle = icon_button(
        ServicesMark::OperationsToggle,
        arrow.size(size::Icon).build_element(),
        move || toggle.call(()),
    )
    .tooltip(if collapsed {
        &props.expand
    } else {
        &props.collapse
    });
    let mut content: Vec<View> = vec![
        Border::new()
            .horizontal_alignment(HorizontalAlignment::Right)
            .content(toggle)
            .into(),
    ];
    if !collapsed {
        let rows: Vec<KeyedView> = props
            .operations
            .iter()
            .map(|op| {
                keyed(
                    op.name.to_string(),
                    Grid::new()
                        .mark(ServicesMark::Operation)
                        .height(Panel::Row)
                        .columns([GridLength::Star(1.0), GridLength::Auto])
                        .children((
                            text(op.display_name.to_string())
                                .text_trimming(TextTrimming::CharacterEllipsis)
                                .vertical_alignment(VerticalAlignment::Center),
                            ProgressRing::new()
                                .mark(ServicesMark::OperationIndicator)
                                .grid_column(1)
                                .width(Panel::Indicator)
                                .height(Panel::Indicator)
                                .is_indeterminate(true),
                        )),
                )
            })
            .collect();
        content.push(
            ScrollViewer::new()
                .mark(ServicesMark::OperationsScroll)
                .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                .vertical_scroll_bar_visibility(if rows.len() > 3 {
                    ScrollBarVisibility::Auto
                } else {
                    ScrollBarVisibility::Disabled
                })
                .max_height(Panel::Row * 3.0)
                .content(StackPanel::new().keyed_children(rows))
                .into(),
        );
    }
    Border::new()
        .mark(SelectionMark::Keeper)
        .content(
            Border::new()
                .mark(ServicesMark::Operations)
                .padding(Thickness::new(
                    space::Header,
                    space::Compact,
                    space::Header,
                    space::Control,
                ))
                .corner_radius(radius::Card)
                .background(palette.menu_fill)
                .border_brush(palette.menu_stroke)
                .border_thickness(1.0)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .vertical_alignment(VerticalAlignment::Bottom)
                .margin(Thickness::new(0.0, 0.0, space::Control, space::Control))
                .opacity(if visible { 1.0 } else { 0.0 })
                .translation(0.0, if visible { 0.0 } else { 24.0 })
                .opacity_transition(std::time::Duration::from_millis(180))
                .translation_transition(std::time::Duration::from_millis(200))
                .content(StackPanel::new().children(content)),
        )
        .into()
}
