use app_contracts::features::agent_link::InProcess;
use guicons::icon;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, Grid, HorizontalAlignment, Orientation, ProgressRing, StackPanel, ThemeBrush, Thickness,
    VerticalAlignment, View,
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
    pub elevated: bool,
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

    let mut layers: Vec<View> = vec![middle.into()];
    if props.in_process_offered {
        let start_in_process = props.on_start_in_process;
        let mut lines: Vec<View> = vec![line(props.l10n.shell_splash_slow(), props.palette).into()];
        match props.service_trouble {
            Some(ServiceTrouble::Unreachable(service)) => lines.push(
                line(props.l10n.shell_splash_unreachable(service.to_string()), props.palette)
                    .mark(SplashMark::Unreachable)
                    .into(),
            ),
            Some(ServiceTrouble::Outdated(service)) => lines.push(
                line(props.l10n.shell_splash_outdated(service.to_string()), props.palette)
                    .mark(SplashMark::Outdated)
                    .into(),
            ),
            None => {}
        }
        match props.in_process {
            InProcess::NotElevated => lines.push(
                line(props.l10n.shell_splash_in_process_not_elevated(), props.palette)
                    .mark(SplashMark::InProcessError)
                    .into(),
            ),
            InProcess::Failed => lines.push(
                line(props.l10n.shell_splash_in_process_failed(), props.palette)
                    .mark(SplashMark::InProcessError)
                    .into(),
            ),
            InProcess::Off | InProcess::Starting | InProcess::Elevating | InProcess::Running => {}
        }
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
            .children(lines);
        let corner = Border::new()
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::xy(space::Section, space::Section))
            .content(action_button(
                SplashMark::OpenInProcess,
                props.l10n.shell_splash_open_in_process(),
                Some(match props.elevated {
                    true => icon!(open).size(size::Icon).build_element(),
                    false => icon!(shield).size(size::Icon).build_element(),
                }),
                !matches!(
                    props.in_process,
                    InProcess::Starting | InProcess::Elevating | InProcess::Running
                ),
                move || start_in_process.call(()),
            ));
        layers.extend([slow.into(), spinner.into(), corner.into()]);
    } else {
        layers.push(spinner.into());
    }

    Border::new()
        .mark(SplashMark::Splash)
        .background(ThemeBrush::SolidBackground)
        .content(Grid::new().children(layers))
        .into()
}
