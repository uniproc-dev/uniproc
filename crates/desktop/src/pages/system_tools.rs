use app_contracts::features::system::SystemState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::system::ToolsProps;
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use crate::routes::Route;

#[derive(Default)]
pub struct SystemTools {
    back_hovered: bool,
}

#[page]
impl Page for SystemTools {
    type Params = crate::routes::SystemToolsParams;
    type Installs = ();
    type Message = bool;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, hovered: bool, _cx: &mut UpdateCx<'_, Self>) {
        self.back_hovered = hovered;
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<SystemState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let on_back_hover = cx.on(|hovered: bool| hovered);
        let nav = cx.navigate::<Route>();
        ui::pages::system::tools_view(ToolsProps {
            state: &state,
            dispatch: &dispatch,
            l10n: &l10n,
            palette,
            back_hovered: self.back_hovered,
            on_back_hover,
            back: Callback::new(move |()| nav.to(Route::System {})),
        })
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::system::SystemTool;
    use guinea::app::Harness;
    use guinea::winui::harness::Mounted;
    use ui::pages::system::{SystemMark, ToolMark};

    use super::*;
    use crate::test_system::{self, launch, launched};

    fn mount(h: &Harness) -> Mounted<'_, SystemTools> {
        let mut page =
            Mounted::mount_at(&h.child(), crate::routes::SystemToolsParams::default(), Route::SystemTools {})
                .unwrap();
        page.settle();
        page
    }

    fn has_pin(page: &Mounted<'_, SystemTools>, tool: SystemTool) -> bool {
        page.tree()
            .find(ToolMark(tool))
            .unwrap_or_else(|| panic!("{tool:?}: {:#?}", page.tree()))
            .find(SystemMark::Pin)
            .is_some()
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn every_tool_has_a_card(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let page = mount(h);
        for tool in SystemTool::ALL {
            assert!(page.find(ToolMark(tool)).is_some(), "{tool:?}: {:#?}", page.tree());
        }
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_found_tool_opens_and_a_missing_one_is_downloaded(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);
        assert!(has_pin(&page, SystemTool::ProcessExplorer));
        assert!(!has_pin(&page, SystemTool::ProcessMonitor), "a missing tool is not pinned");
        assert!(page.find(SystemMark::Forget).is_none(), "the catalog has nothing to take off");

        for tool in [
            SystemTool::ProcessExplorer,
            SystemTool::ProcessMonitor,
            SystemTool::ReliabilityMonitor,
            SystemTool::EnvironmentVariables,
            SystemTool::StartupApps,
        ] {
            page.within(ToolMark(tool)).click(SystemMark::Open).settle();
        }

        assert_eq!(
            launched(),
            [
                launch(r"C:\Tools\procexp64.exe", ""),
                launch("https://learn.microsoft.com/sysinternals/downloads/procmon", ""),
                launch("perfmon.exe", "/rel"),
                launch("rundll32.exe", "sysdm.cpl,EditEnvironmentVariables"),
                launch("ms-settings:startupapps", ""),
            ]
        );
        let uses = test_system::stored(h).uses().get(SystemTool::ProcessExplorer.id()).unwrap();
        assert_eq!(uses.count, 1);
        assert!(test_system::stored(h).uses().get(SystemTool::ProcessMonitor.id()).is_none(), "a download is not a use");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_pin_is_kept(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);

        page.within(ToolMark(SystemTool::DeviceManager)).click(SystemMark::Pin).settle();
        page.settle();

        assert!(h.state::<SystemState>().is_pinned(SystemTool::DeviceManager));
        assert!(test_system::stored(h).pinned().get(SystemTool::DeviceManager.id()).is_some());
        assert!(launched().is_empty(), "pinning does not open the tool");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_crumb_goes_back_to_system(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);

        page.click(SystemMark::Back).settle();

        assert_eq!(page.navigated::<Route>(), [Route::System {}]);
    }
}
