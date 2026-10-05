use guinea::prelude::*;

use crate::app::App;
use crate::layouts::{MainWindow, Overlay, PaneFooter, ProcessesArea, Shell, SystemArea};
use crate::pages::{Activity, Processes, ProcessesSettings, Services, Settings, System, SystemTools, Wsl};
use crate::parts::{Connecting, SidebarCharts};

routes! {
    Route {
        app(App) {
            layout(MainWindow) restorable {
                layout(Shell) {
                    part(SidebarCharts) => PaneFooter
                    part(Connecting) => Overlay
                    layout(ProcessesArea) keep {
                        page(Processes)
                        page(ProcessesSettings)
                    }
                    page(Activity)
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
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use app_contracts::features::agents::{
        AgentStateRequest, WindowsMachineSample, WindowsMachineStats, WindowsProcessStats, WindowsReport,
    };
    use guinea::core::trace::Point;
    use domain::features::agent_link::AgentLinkDeps;
    use domain::features::processes::windows_scan::AppWindows;
    use domain::features::processes::ProcessesDeps;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};

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

    fn start(h: &mut Harness) {
        test_agent::reset(true);
        h.install_application_with(crate::app::with_fakes)
            .unwrap()
            .provide(AgentLinkDeps {
                start_in_process: test_agent::start_in_process,
            })
            .provide(ProcessesDeps {
                windows: AppWindows::default,
                shell: |_| {},
            });
    }

    fn sample(clock_100ns: u64) -> WindowsMachineSample {
        WindowsMachineSample {
            machine: std::sync::Arc::new(WindowsMachineStats::default()),
            clock_100ns,
        }
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn a_machine_sample_redraws_the_sidebar_charts_and_not_the_shell_or_the_page(h: &mut Harness) {
        start(h);
        let h = &*h;
        serve("notepad.exe");
        let mut app = Mounted::routed(h, Route::Services {}).unwrap();
        h.advance(Duration::from_secs(1));
        app.settle();

        h.publish(sample(0)).settle();
        app.settle();
        let act = h.publish(sample(10_000_000));
        act.settle();
        app.settle();
        let drawn: Vec<&str> = act
            .chain()
            .points()
            .into_iter()
            .filter_map(|point| match point {
                Point::Render { segment, .. } => Some(*segment),
                _ => None,
            })
            .collect();

        assert!(drawn.iter().any(|segment| segment.ends_with("SidebarCharts")), "{drawn:?}");
        assert!(!drawn.iter().any(|segment| segment.ends_with("Shell")), "{drawn:?}");
        assert!(!drawn.iter().any(|segment| segment.ends_with("Services")), "{drawn:?}");
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn processes_come_back_as_they_were_left_and_catch_up_after(h: &mut Harness) {
        start(h);
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
