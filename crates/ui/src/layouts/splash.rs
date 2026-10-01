use app_contracts::features::agent_link::InProcess;
use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, ChildrenControl, ContentControl, Grid, HorizontalAlignment, LayoutControl,
    Orientation, ProgressRing, StackPanel, ThemeBrush, Thickness, VerticalAlignment, View,
};

use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::button::action_button;
use crate::widgets::text::{subtitle, text};

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplashMark {
    Splash,
    OpenInProcess,
    Unreachable,
    Outdated,
    InProcessError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceTrouble<'a> {
    Unreachable(&'a str),
    Outdated(&'a str),
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
    pub in_process_offered: bool,
    pub in_process: InProcess,
    pub service_trouble: Option<ServiceTrouble<'a>>,
    pub on_start_in_process: Callback<()>,
}

fn line(content: String, palette: Palette) -> windows_reactor::TextBlock {
    text(content)
        .foreground(palette.secondary_text)
        .horizontal_alignment(HorizontalAlignment::Center)
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

    let (slow, corner): (View, View) = if props.in_process_offered {
        let start_in_process = props.on_start_in_process;
        let trouble: View = match props.service_trouble {
            Some(ServiceTrouble::Unreachable(service)) => {
                line(props.l10n.shell_splash_unreachable(service.to_string()), props.palette)
                    .mark(SplashMark::Unreachable)
                    .into()
            }
            Some(ServiceTrouble::Outdated(service)) => {
                line(props.l10n.shell_splash_outdated(service.to_string()), props.palette)
                    .mark(SplashMark::Outdated)
                    .into()
            }
            None => View::empty(),
        };
        let in_process_error: View = match props.in_process {
            InProcess::NotElevated => line(props.l10n.shell_splash_in_process_not_elevated(), props.palette)
                .mark(SplashMark::InProcessError)
                .into(),
            InProcess::Failed => line(props.l10n.shell_splash_in_process_failed(), props.palette)
                .mark(SplashMark::InProcessError)
                .into(),
            InProcess::Off | InProcess::Starting | InProcess::Running => View::empty(),
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
                line(props.l10n.shell_splash_slow(), props.palette),
                trouble,
                in_process_error,
            ))
            .into();
        let corner = Border::new()
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::xy(space::Section, space::Section))
            .content(action_button(
                SplashMark::OpenInProcess,
                props.l10n.shell_splash_open_in_process(),
                Some(icon!(open).size(size::Icon).build_element()),
                !matches!(props.in_process, InProcess::Starting | InProcess::Running),
                move || {
                    let _ = start_in_process.call(());
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
