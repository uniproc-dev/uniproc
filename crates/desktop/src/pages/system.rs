use app_contracts::features::system::SystemState;
use domain::features::system::{SystemDeps, SystemFeature};
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct System;

#[page]
impl Page for System {
    type Params = crate::routes::SystemParams;
    type Installs = SystemFeature;
    type Message = ();

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        let deps = ctx.require_or_default::<SystemDeps>();
        ctx.install(&deps)
    }

    fn update(&mut self, _message: (), _cx: &mut UpdateCx<'_, Self>) {}

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<SystemState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        ui::pages::system::system_view(&state, &dispatch, &l10n, palette)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use app_contracts::features::system::SystemTool;
    use domain::features::system::tools::Launch;
    use guinea::app::Harness;
    use guinea::winui::harness::Mounted;
    use guinea_plugin_l10n::L10nPlugin;
    use ui::pages::system::ToolMark;

    use super::*;

    static LAUNCHED: Mutex<Vec<Launch>> = Mutex::new(Vec::new());

    fn launched() -> Vec<Launch> {
        LAUNCHED.lock().unwrap().clone()
    }

    fn record(launch: Launch) {
        LAUNCHED.lock().unwrap().push(launch);
    }

    fn procexp_only(name: &str) -> Option<PathBuf> {
        (name == "procexp64.exe").then(|| PathBuf::from(r"C:\Tools\procexp64.exe"))
    }

    fn mount(h: &mut Harness) -> Mounted<'_, System> {
        LAUNCHED.lock().unwrap().clear();
        h.plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(SystemDeps {
                locate: procexp_only,
                launch: record,
            });
        let h = &*h;
        let mut page = Mounted::<System>::mount(&h.child(), crate::routes::SystemParams::default()).unwrap();
        page.settle();
        page
    }

    fn button_says(page: &Mounted<'_, System>, tool: SystemTool, label: &str) -> bool {
        page.tree()
            .find(ToolMark(tool))
            .unwrap_or_else(|| panic!("{tool:?}: {:#?}", page.tree()))
            .find_text(label)
            .is_some()
    }

    #[guinea::test(iterations = 4, exclusive = "system_tools")]
    fn every_tool_has_a_row(h: &mut Harness) {
        let page = mount(h);
        for tool in SystemTool::ALL {
            assert!(page.find(ToolMark(tool)).is_some(), "{tool:?}: {:#?}", page.tree());
        }
    }

    #[guinea::test(iterations = 4, exclusive = "system_tools")]
    fn a_found_tool_opens_and_a_missing_one_is_downloaded(h: &mut Harness) {
        let mut page = mount(h);
        assert!(button_says(&page, SystemTool::ProcessExplorer, "Open"));
        assert!(button_says(&page, SystemTool::ProcessMonitor, "Download"));
        assert!(button_says(&page, SystemTool::EventViewer, "Open"));

        page.click(ToolMark(SystemTool::ProcessExplorer)).settle();
        page.click(ToolMark(SystemTool::ProcessMonitor)).settle();
        page.click(ToolMark(SystemTool::ReliabilityMonitor)).settle();
        page.click(ToolMark(SystemTool::EnvironmentVariables)).settle();

        let launch = |file: &str, args: &str| Launch {
            file: file.into(),
            args: args.into(),
        };
        assert_eq!(
            launched(),
            [
                launch(r"C:\Tools\procexp64.exe", ""),
                launch("https://learn.microsoft.com/sysinternals/downloads/procmon", ""),
                launch("perfmon.exe", "/rel"),
                launch("rundll32.exe", "sysdm.cpl,EditEnvironmentVariables"),
            ]
        );
    }
}
