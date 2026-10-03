use app_contracts::features::activity::ActivityState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::activity::{ActivityPage, ActivityPageMsg};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Activity(ActivityPage);

#[page]
impl Page for Activity {
    type Params = crate::routes::ActivityParams;
    type Installs = ();
    type Message = ActivityPageMsg;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, message: ActivityPageMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.read::<ActivityState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ActivityPageMsg| message);
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::activity::Clock;
    use app_contracts::features::agents::{
        ProcessCame, ProcessEvent, ProcessInstance, ProcessWent, WindowsProcessEvents,
    };
    use domain::features::activity::{ActivityDeps, ActivityFeature};
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::activity::ActivityMark;

    use super::*;
    use crate::routes::Route;

    const SECOND: u64 = 10_000_000;
    const HOUR: u64 = 3600 * SECOND;
    const BASE: u64 = 1000 * HOUR;

    fn utc(at: u64) -> Clock {
        let seconds = at / SECOND;
        Clock {
            hour: (seconds / 3600 % 24) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
        }
    }

    fn start(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ActivityDeps {
                now: || BASE + HOUR - SECOND,
                clock: utc,
            });
        h.feature(ActivityFeature).unwrap();
    }

    fn mount(h: &Harness) -> Mounted<'_, Activity> {
        let mut page =
            Mounted::mount_at(h.child(), crate::routes::ActivityParams::default(), Route::Activity {}).unwrap();
        page.settle();
        page
    }

    fn id(pid: u32) -> ProcessInstance {
        ProcessInstance { pid, sequence: 7 }
    }

    fn came(pid: u32, minute: u64) -> ProcessEvent {
        ProcessEvent::Came(ProcessCame {
            instance: id(pid),
            parent: id(1),
            at: BASE + minute * 60 * SECOND,
            image_path: r"C:\Tools\tool.exe".into(),
            command_line: "tool.exe --check".into(),
            ..Default::default()
        })
    }

    fn went(pid: u32, minute: u64) -> ProcessEvent {
        ProcessEvent::Went(ProcessWent {
            instance: id(pid),
            at: BASE + minute * 60 * SECOND,
            ..Default::default()
        })
    }

    fn says(node: &Node, wanted: &str) -> bool {
        node.text.as_deref().map(|text| text.replace(['\u{2068}', '\u{2069}'], "")).as_deref() == Some(wanted)
            || node.children.iter().any(|child| says(child, wanted))
    }

    fn live(h: &Harness, page: &mut Mounted<'_, Activity>, events: Vec<ProcessEvent>) {
        h.publish(WindowsProcessEvents {
            events: Arc::from(events),
            ..WindowsProcessEvents::default()
        })
        .settle();
        page.settle();
    }

    fn rows(node: &Node) -> usize {
        usize::from(node.id.as_deref() == Some("Row")) + node.children.iter().map(rows).sum::<usize>()
    }

    #[guinea::test(iterations = 4)]
    fn a_process_that_came_is_listed_by_name_and_its_command_line_waits_in_its_facts(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);

        let tree = page.tree();
        assert!(says(&tree, "tool.exe"), "{tree:#?}");
        assert!(!says(&tree, "tool.exe --check"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn a_row_opens_to_its_facts(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Facts).is_none());

        page.click(ActivityMark::Row).settle();
        page.settle();

        let tree = page.tree();
        assert!(page.find(ActivityMark::Facts).is_some(), "{tree:#?}");
        assert!(says(&tree, "tool.exe --check"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn an_exit_of_a_process_never_seen_says_so_instead_of_a_bare_number(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![went(30, 20)]);

        let tree = page.tree();
        assert!(says(&tree, "Process 30"), "{tree:#?}");
        assert!(!says(&tree, "30"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn hiding_exits_from_the_menu_leaves_nothing_when_only_exits_happened(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![went(30, 20)]);
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Menu).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::Went).settle();
        page.settle();

        assert!(page.find(ActivityMark::Empty).is_some(), "{:#?}", page.tree());
    }
}
