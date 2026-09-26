use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, ChildrenControl, ContentControl, Grid, HorizontalAlignment, LayoutControl,
    Orientation, ProgressRing, StackPanel, ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::page::action_button;
use crate::widgets::text::{subtitle, text};

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplashMark {
    Splash,
    OpenNative,
    Unreachable,
}

struct Splash;

#[expect(non_upper_case_globals)]
impl Splash {
    const Logo: f64 = 118.0;
    const Spinner: f64 = 32.0;
    const SpinnerFromBottom: f64 = 96.0;
}

pub struct SplashProps<'a> {
    pub l10n: &'a L10n,
    pub palette: Palette,
    pub native_offered: bool,
    pub unreachable_service: Option<&'a str>,
    pub on_open_native: Callback<()>,
}

pub fn splash_view(props: SplashProps<'_>) -> View {
    let middle = StackPanel::new()
        .orientation(Orientation::Vertical)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .children((
            Border::new()
                .horizontal_alignment(HorizontalAlignment::Center)
                .content(icon!(uniproc_logo_splash).size(Splash::Logo).build_element()),
            subtitle(props.l10n.shell_splash_name())
                .foreground(props.palette.secondary_text)
                .horizontal_alignment(HorizontalAlignment::Center),
        ));

    let spinner = ProgressRing::new()
        .is_indeterminate(true)
        .width(Splash::Spinner)
        .height(Splash::Spinner)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Bottom)
        .margin(Thickness::new(0.0, 0.0, 0.0, Splash::SpinnerFromBottom));

    let (slow, corner): (View, View) = if props.native_offered {
        let open_native = props.on_open_native;
        let unreachable: View = match props.unreachable_service {
            Some(service) => text(props.l10n.shell_splash_unreachable(service.to_string()))
                .mark(SplashMark::Unreachable)
                .foreground(props.palette.secondary_text)
                .horizontal_alignment(HorizontalAlignment::Center)
                .into(),
            None => View::empty(),
        };
        let slow = StackPanel::new()
            .orientation(Orientation::Vertical)
            .spacing(space::Compact)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::new(
                0.0,
                0.0,
                0.0,
                Splash::SpinnerFromBottom + Splash::Spinner + space::Card,
            ))
            .children((
                text(props.l10n.shell_splash_slow())
                    .foreground(props.palette.secondary_text)
                    .horizontal_alignment(HorizontalAlignment::Center),
                unreachable,
            ))
            .into();
        let corner = Border::new()
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::xy(space::Section, space::Section))
            .content(action_button(
                SplashMark::OpenNative,
                props.l10n.shell_splash_open_native(),
                Some(icon!(open).size(size::Icon).build_element()),
                move || {
                    let _ = open_native.call(());
                },
            ))
            .into();
        (slow, corner)
    } else {
        (View::empty(), View::empty())
    };

    Border::new()
        .mark(SplashMark::Splash)
        .background(ThemeBrush::SolidBackground)
        .content(Grid::new().children((middle, slow, spinner, corner)))
        .into()
}
