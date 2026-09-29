use std::time::Duration;

use app_contracts::features::system::SystemTool;

use super::settings::ToolUse;

pub(super) struct Recent;

#[expect(non_upper_case_globals)]
impl Recent {
    const Window: Duration = Duration::from_secs(30 * 24 * 60 * 60);
    const Shown: usize = 6;

    fn window_ms() -> u64 {
        Self::Window.as_millis() as u64
    }
}

pub(super) fn pinned(pins: impl IntoIterator<Item = (String, u64)>) -> Vec<SystemTool> {
    let mut pins: Vec<(SystemTool, u64)> = pins
        .into_iter()
        .filter_map(|(id, order)| SystemTool::from_id(&id).map(|tool| (tool, order)))
        .collect();
    pins.sort_by_key(|(_, order)| *order);
    pins.into_iter().map(|(tool, _)| tool).collect()
}

pub(super) fn frequent(
    uses: impl IntoIterator<Item = (String, ToolUse)>,
    pinned: &[SystemTool],
    now_ms: u64,
) -> Vec<SystemTool> {
    let mut used: Vec<(SystemTool, ToolUse)> = uses
        .into_iter()
        .filter_map(|(id, used)| SystemTool::from_id(&id).map(|tool| (tool, used)))
        .filter(|(tool, used)| {
            used.count > 0 && !pinned.contains(tool) && now_ms.saturating_sub(used.last_ms) <= Recent::window_ms()
        })
        .collect();
    used.sort_by(|(_, a), (_, b)| b.count.cmp(&a.count).then(b.last_ms.cmp(&a.last_ms)));
    used.into_iter().take(Recent::Shown).map(|(tool, _)| tool).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn used(tool: SystemTool, count: u32, last_ms: u64) -> (String, ToolUse) {
        (tool.id().to_string(), ToolUse { count, last_ms })
    }

    #[test]
    fn pins_keep_the_order_they_were_made_in() {
        let pins = [
            (SystemTool::Autoruns.id().to_string(), 7),
            ("gone".to_string(), 1),
            (SystemTool::EventViewer.id().to_string(), 3),
        ];
        assert_eq!(pinned(pins), [SystemTool::EventViewer, SystemTool::Autoruns]);
    }

    #[test]
    fn the_most_used_come_first_and_the_last_used_breaks_a_tie() {
        let now = Recent::window_ms() * 2;
        let uses = [
            used(SystemTool::Services, 2, now - 10),
            used(SystemTool::RegistryEditor, 5, now - 10),
            used(SystemTool::EventViewer, 2, now - 1),
        ];
        assert_eq!(
            frequent(uses, &[], now),
            [SystemTool::RegistryEditor, SystemTool::EventViewer, SystemTool::Services]
        );
    }

    #[test]
    fn pinned_and_long_unused_tools_are_not_frequent() {
        let now = Recent::window_ms() * 2;
        let uses = [
            used(SystemTool::Services, 9, now - Recent::window_ms() - 1),
            used(SystemTool::RegistryEditor, 5, now),
            used(SystemTool::EventViewer, 1, now),
        ];
        assert_eq!(frequent(uses, &[SystemTool::RegistryEditor], now), [SystemTool::EventViewer]);
    }

    #[test]
    fn only_a_handful_are_shown() {
        let uses = SystemTool::ALL.map(|tool| used(tool, 1, 0));
        assert_eq!(frequent(uses, &[], 0).len(), Recent::Shown);
    }
}
