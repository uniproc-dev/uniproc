use app_contracts::features::system::SystemState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use crate::routes::Route;

#[derive(Default)]
pub struct System;

#[page]
impl Page for System {
    type Params = crate::routes::SystemParams;
    type Installs = ();
    type Message = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, _message: (), _cx: &mut UpdateCx<'_, Self>) {}

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<SystemState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let nav = cx.navigate::<Route>();
        let open_tools = Callback::new(move |()| nav.to(Route::SystemTools {}));
        ui::pages::system::system_view(&state, &dispatch, &l10n, palette, open_tools)
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::system::{OpenTool, PinTool, SystemTool};
    use domain::features::system::settings::SystemSettings;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};
    use ui::pages::system::{SystemMark, ToolMark};

    use super::*;
    use crate::test_system::{self, launch, launched};

    fn mount(h: &Harness) -> Mounted<'_, System> {
        let mut page =
            Mounted::mount_at(&h.child(), crate::routes::SystemParams::default(), Route::System {}).unwrap();
        page.settle();
        page
    }

    fn tools_in(node: &Node, found: &mut Vec<SystemTool>) {
        if let Some(tool) = node.id.as_deref().and_then(SystemTool::from_id) {
            found.push(tool);
        }
        for child in &node.children {
            tools_in(child, found);
        }
    }

    fn favourites(page: &Mounted<'_, System>) -> Vec<SystemTool> {
        let tree = page.tree();
        let mut found = Vec::new();
        if let Some(shown) = tree.find(SystemMark::Favourites) {
            tools_in(shown, &mut found);
        }
        found
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_tools_card_opens_the_tools_page(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);
        assert!(page.find(SystemMark::NoFavourites).is_some(), "{:#?}", page.tree());

        page.click(SystemMark::Tools).settle();

        assert_eq!(page.navigated::<Route>(), [Route::SystemTools {}]);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn pinned_tools_come_first_then_the_most_opened(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);

        for tool in [SystemTool::EventViewer, SystemTool::RegistryEditor, SystemTool::EventViewer] {
            h.act::<SystemState>(OpenTool(tool)).settle();
        }
        h.act::<SystemState>(PinTool(SystemTool::ProcessExplorer, true)).settle();
        page.settle();

        assert_eq!(
            favourites(&page),
            [SystemTool::ProcessExplorer, SystemTool::EventViewer, SystemTool::RegistryEditor],
            "{:#?}",
            page.tree()
        );
        assert_eq!(
            launched(),
            [
                launch("eventvwr.msc", ""),
                launch("regedit.exe", ""),
                launch("eventvwr.msc", ""),
            ]
        );

        page.within(ToolMark(SystemTool::ProcessExplorer)).click(SystemMark::Pin).settle();
        page.settle();
        assert_eq!(favourites(&page), [SystemTool::EventViewer, SystemTool::RegistryEditor]);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_cross_takes_a_tool_off_the_list(h: &mut Harness) {
        test_system::start(h);
        let h = &*h;
        let mut page = mount(h);
        for tool in [SystemTool::EventViewer, SystemTool::EventViewer, SystemTool::Services] {
            h.act::<SystemState>(OpenTool(tool)).settle();
        }
        h.act::<SystemState>(PinTool(SystemTool::ProcessExplorer, true)).settle();
        page.settle();
        assert_eq!(
            favourites(&page),
            [SystemTool::ProcessExplorer, SystemTool::EventViewer, SystemTool::Services]
        );

        page.within(ToolMark(SystemTool::ProcessExplorer)).click(SystemMark::Forget).settle();
        page.within(ToolMark(SystemTool::EventViewer)).click(SystemMark::Forget).settle();
        page.settle();

        assert_eq!(favourites(&page), [SystemTool::Services]);
        assert_eq!(launched().len(), 3, "the cross opens nothing");
        let stored = SystemSettings::new().unwrap();
        assert!(stored.pinned().get(SystemTool::ProcessExplorer.id()).is_none());
        assert!(stored.uses().get(SystemTool::EventViewer.id()).is_none());
    }
}
