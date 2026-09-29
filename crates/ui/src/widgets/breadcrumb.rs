use guicons::icon;
use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Border, Callback, ChildrenControl, Color, ContentControl, Grid, LayoutControl, Orientation, PointerEventInfo,
    StackPanel, ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::theme::{space, Palette};
use crate::widgets::text::subtitle;

struct Crumb;

#[expect(non_upper_case_globals)]
impl Crumb {
    const ChevronSize: f64 = 12.0;
    const ChevronLead: f64 = 2.0;
    const ChevronTrail: f64 = 1.0;
    const ChevronDrop: f64 = 2.0;
    const Hit: Color = Color::argb(0, 0, 0, 0);
}

pub struct Breadcrumb {
    pub parent: String,
    pub current: String,
    pub hovered: bool,
}

pub fn breadcrumb(
    back_mark: impl Mark,
    crumb: Breadcrumb,
    palette: Palette,
    on_hover: Callback<bool>,
    back: Callback<()>,
) -> View {
    let Breadcrumb { parent, current, hovered } = crumb;
    let (entered, exited) = (on_hover.clone(), on_hover);
    let parent_text = subtitle(parent);
    let parent_text = if hovered {
        parent_text.foreground(ThemeBrush::PrimaryText)
    } else {
        parent_text.foreground(palette.secondary_text)
    };
    let parent = Border::new()
        .mark(back_mark)
        .background(Crumb::Hit)
        .vertical_alignment(VerticalAlignment::Center)
        .on_pointer_entered(move |_: PointerEventInfo| {
            let _ = entered.call(true);
        })
        .on_pointer_exited(move |_: PointerEventInfo| {
            let _ = exited.call(false);
        })
        .on_pointer_released(move |_: PointerEventInfo| {
            let _ = back.call(());
        })
        .content(parent_text);
    let chevron = Grid::new()
        .margin(Thickness::new(Crumb::ChevronLead, Crumb::ChevronDrop, Crumb::ChevronTrail, 0.0))
        .vertical_alignment(VerticalAlignment::Center)
        .children((icon!(chevron_right_regular).size(Crumb::ChevronSize).build_element(),));
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Compact)
        .children((
            parent,
            chevron,
            subtitle(current).vertical_alignment(VerticalAlignment::Center),
        ))
        .into()
}
