use guinea::winui::MarkExt;
use guinea::Mark;
use windows_reactor::{
    Border, Button, ButtonStyle, Orientation, ResourceOverrides, StackPanel, Thickness, View,
};

use crate::theme::{opacity, space};
use crate::widgets::text::text;

pub fn command_button(
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    labelled_button(Some(ButtonStyle::Subtle), mark, label, icon, enabled, on_click)
}

struct IconButton;

#[expect(non_upper_case_globals)]
impl IconButton {
    const Padding: f64 = 6.0;
}

pub fn icon_button(mark: impl Mark, icon: View, on_click: impl Fn() + 'static) -> View {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .resource_overrides(ResourceOverrides::new().set("ButtonPadding", Thickness::uniform(IconButton::Padding)))
        .on_click(on_click)
        .content(icon)
        .into()
}

pub fn action_button(
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    labelled_button(None, mark, label, icon, enabled, on_click)
}

fn labelled_button(
    style: Option<ButtonStyle>,
    mark: impl Mark,
    label: impl Into<String>,
    icon: Option<View>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    let button = Button::new().mark(mark).is_enabled(enabled).on_click(on_click);
    let button = match style {
        Some(style) => button.style(style),
        None => button,
    };
    match icon {
        Some(icon) => button.content(
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(space::Control)
                .children((
                    Border::new()
                        .opacity(if enabled { 1.0 } else { opacity::Disabled })
                        .content(icon),
                    text(label),
                )),
        ),
        None => button.content(text(label)),
    }
    .into()
}
