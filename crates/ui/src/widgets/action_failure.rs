use app_contracts::features::agents::{ActionFailure, ActionOutcome, WindowsAction};
use guinea::winui::MarkExt;
use windows_reactor::{TeachingTip, View};

use crate::l10n::L10n;

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionFailureMark {
    Tip,
}

fn action_id(action: &WindowsAction) -> &'static str {
    match action {
        WindowsAction::Kill { .. } => "kill",
        WindowsAction::Suspend { .. } => "suspend",
        WindowsAction::Resume { .. } => "resume",
        WindowsAction::ServiceStart { .. } => "start",
        WindowsAction::ServiceStop { .. } => "stop",
        WindowsAction::ServicePause { .. } => "pause",
        WindowsAction::ServiceResume { .. } => "continue",
        WindowsAction::ServiceRestart { .. } => "restart",
        WindowsAction::SetPriority { .. } | WindowsAction::SetAffinity { .. } => "other",
    }
}

fn reason(outcome: ActionOutcome, l10n: &L10n) -> String {
    match outcome {
        ActionOutcome::Denied => l10n.action_failure_denied(),
        ActionOutcome::Gone => l10n.action_failure_gone(),
        ActionOutcome::Busy => l10n.action_failure_busy(),
        ActionOutcome::NotConnected => l10n.action_failure_not_connected(),
        ActionOutcome::Failed(code) => l10n.action_failure_failed(code.to_string()),
        ActionOutcome::Done => String::new(),
    }
}

pub fn action_failure(
    failure: Option<&ActionFailure>,
    l10n: &L10n,
    dismiss: impl Fn() + 'static,
) -> View {
    let (title, subtitle) = match failure {
        Some(failure) => (
            l10n.action_failure_title(action_id(&failure.action), failure.target.to_string()),
            reason(failure.outcome, l10n),
        ),
        None => (String::new(), String::new()),
    };
    TeachingTip::new()
        .mark(ActionFailureMark::Tip)
        .is_open(failure.is_some())
        .is_light_dismiss_enabled(true)
        .title(title)
        .subtitle(subtitle)
        .on_closed(dismiss)
        .into()
}
