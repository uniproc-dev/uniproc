use guinea::prelude::*;

use crate::layouts::{ProcessesArea, ShellLayout, SystemArea};
use crate::pages::{Processes, ProcessesSettings, Services, Settings, System, SystemTools, Wsl};

routes! {
    Route {
        layout(ShellLayout) restorable {
            layout(ProcessesArea) keep {
                page(Processes)
                page(ProcessesSettings)
            }
            page(Services)
            page(Wsl)
            layout(SystemArea) {
                page(System)
                page(SystemTools)
            }
            page(Settings)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use app_contracts::features::agents::{AgentStateRequest, WindowsProcessStats, WindowsReport};
    use domain::features::agent_link::AgentLinkDeps;
    use domain::features::processes::windows_scan::AppWindows;
    use domain::features::processes::ProcessesDeps;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;

    use super::*;
    use crate::test_agent;

    fn serve(name: &str) {
        test_agent::serve(WindowsReport {
            processes: vec![WindowsProcessStats { pid: 10, name: name.into(), ..Default::default() }],
            ..Default::default()
        });
    }

    fn says(node: &Node, wanted: &str) -> bool {
        node.text.as_deref() == Some(wanted) || node.children.iter().any(|child| says(child, wanted))
    }

    fn listed(app: &mut Mounted<'_, Route>, name: &str) -> bool {
        app.items().iter().any(|item| says(item, name))
    }

    fn start(h: &mut Harness) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        test_agent::reset(true);
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .feature(test_agent::FakeAgentFeature)
            .unwrap()
            .provide(AgentLinkDeps {
                start_in_process: test_agent::start_in_process,
            })
            .provide(ProcessesDeps {
                windows: AppWindows::default,
                shell: |_| {},
            });
        dir
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn processes_come_back_as_they_were_left_and_catch_up_after(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        serve("notepad.exe");
        let mut app = Mounted::routed(h, Route::Processes {}).unwrap();
        h.advance(Duration::from_secs(1));
        app.settle();
        assert!(listed(&mut app, "notepad.exe"), "{:#?}", app.tree());

        app.navigate(Route::Services {});
        assert!(!app.is_mounted::<ProcessesArea>());
        serve("calc.exe");
        h.advance(Duration::from_secs(1));
        app.settle();

        let back = app.navigate(Route::Processes {});
        assert!(back.chain().published::<AgentStateRequest>(), "{:#?}", back.chain());
        assert!(listed(&mut app, "notepad.exe"), "kept rows show at once: {:#?}", app.tree());
        assert!(!listed(&mut app, "calc.exe"), "a report sent while asleep is not applied");

        h.advance(Duration::from_secs(1));
        app.settle();
        assert!(listed(&mut app, "calc.exe"), "{:#?}", app.tree());
    }
}
