#[derive(Clone, PartialEq, Debug, guinea::Routable)]
pub enum Route {
    Processes(ProcessesRoute),
    Activity(ActivityRoute),
    Services {},
    Wsl {},
    System(SystemRoute),
    Settings {},
}

#[derive(Default, Clone, PartialEq, Debug, guinea::Routable)]
pub enum ProcessesRoute {
    #[default]
    Overview,
    Settings,
}

#[derive(Default, Clone, PartialEq, Debug, guinea::Routable)]
pub enum ActivityRoute {
    #[default]
    Overview,
    Groups,
}

#[derive(Default, Clone, PartialEq, Debug, guinea::Routable)]
pub enum SystemRoute {
    #[default]
    Overview,
    Tools,
}

impl Route {
    pub fn save(&self) -> Option<String> {
        Some(match self {
            Self::Processes(ProcessesRoute::Overview) => "Processes",
            Self::Processes(ProcessesRoute::Settings) => "ProcessesSettings",
            Self::Activity(ActivityRoute::Overview) => "Activity",
            Self::Activity(ActivityRoute::Groups) => "ActivityGroups",
            Self::Services {} => "Services",
            Self::Wsl {} => "Wsl",
            Self::System(SystemRoute::Overview) => "System",
            Self::System(SystemRoute::Tools) => "SystemTools",
            Self::Settings {} => "Settings",
        }.to_owned())
    }

    pub fn restore(saved: &str) -> Option<Self> {
        Some(match saved {
            "Processes" => Self::Processes(ProcessesRoute::Overview),
            "ProcessesSettings" => Self::Processes(ProcessesRoute::Settings),
            "Activity" => Self::Activity(ActivityRoute::Overview),
            "ActivityGroups" => Self::Activity(ActivityRoute::Groups),
            "Services" => Self::Services {},
            "Wsl" => Self::Wsl {},
            "System" => Self::System(SystemRoute::Overview),
            "SystemTools" => Self::System(SystemRoute::Tools),
            "Settings" => Self::Settings {},
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use app_contracts::features::agents::{
        AgentStateRequest, WindowsMachineSample, WindowsMachineStats, WindowsProcessStats, WindowsReport,
    };
    use guinea::observability::trace::Point;
    use domain::features::processes::windows_scan::AppWindows;
    use domain::features::processes::ProcessesDeps;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};

    use super::*;
    use crate::layouts::ProcessesArea;
    use crate::pages::{Services, Settings};
    use crate::test_agent;

    #[test]
    fn saved_routes_round_trip() {
        for route in [
            Route::Processes(ProcessesRoute::Overview),
            Route::Processes(ProcessesRoute::Settings),
            Route::Activity(ActivityRoute::Overview),
            Route::Activity(ActivityRoute::Groups),
            Route::Services {},
            Route::Wsl {},
            Route::System(SystemRoute::Overview),
            Route::System(SystemRoute::Tools),
            Route::Settings {},
        ] {
            assert_eq!(Route::restore(&route.save().unwrap()), Some(route));
        }
        assert_eq!(Route::restore(""), None);
        assert_eq!(Route::restore("unknown"), None);
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn nested_deep_links_mount_their_page(h: &mut Harness) {
        start(h);
        for route in [
            Route::Processes(ProcessesRoute::Settings),
            Route::Activity(ActivityRoute::Groups),
            Route::System(SystemRoute::Tools),
        ] {
            let mut app = crate::test_support::mount_application(h, route.clone()).unwrap();
            app.settle();
            assert_eq!(app.route(), route);
            assert_eq!(app.is_mounted::<crate::pages::ProcessesSettings>(), matches!(route, Route::Processes(_)));
            assert_eq!(app.is_mounted::<crate::pages::ActivityGroups>(), matches!(route, Route::Activity(_)));
            assert_eq!(app.is_mounted::<crate::pages::SystemTools>(), matches!(route, Route::System(_)));
        }
    }

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
        h.app(|app| crate::app::with_fakes(app, crate::platform::PlatformDeps {
            processes: ProcessesDeps {
                windows: AppWindows::default,
                shell: |_| {},
            },
            agent_link: test_agent::agent_link(),
            ..Default::default()
        })).unwrap();
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
        let mut app = crate::test_support::mount_application(h, Route::Services {}).unwrap();
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
    fn a_page_left_with_a_redraw_pending_is_not_drawn_again(h: &mut Harness) {
        start(h);
        let h = &*h;
        serve("notepad.exe");
        let mut app = crate::test_support::mount_application(h, Route::Services {}).unwrap();
        h.advance(Duration::from_secs(1));
        app.settle();
        serve("calc.exe");
        h.advance(Duration::from_secs(1));

        let left = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            app.navigate(Route::Settings {});
            app.settle();
        }));

        assert!(left.is_ok(), "leaving Services with a redraw pending panicked");
        assert!(app.is_mounted::<Settings>());
        assert!(!app.is_mounted::<Services>());
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn a_press_that_leaves_activity_does_not_reach_the_page_it_left(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut app = crate::test_support::mount_application(h, Route::Activity(ActivityRoute::Overview)).unwrap();
        app.settle();

        let left = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            h.publish(app_contracts::features::window::PressedAway);
            app.navigate(Route::Services {});
            app.settle();
        }));

        assert!(left.is_ok(), "a press away that left Activity panicked");
        assert!(app.is_mounted::<Services>());
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn processes_come_back_as_they_were_left_and_catch_up_after(h: &mut Harness) {
        start(h);
        let h = &*h;
        serve("notepad.exe");
        let mut app = crate::test_support::mount_application(h, Route::Processes(ProcessesRoute::Overview)).unwrap();
        h.advance(Duration::from_secs(1));
        app.settle();
        assert!(listed(&mut app, "notepad.exe"), "{:#?}", app.tree());

        app.navigate(Route::Services {});
        assert!(!app.is_mounted::<ProcessesArea>());
        serve("calc.exe");
        h.advance(Duration::from_secs(1));
        app.settle();

        let back = app.navigate(Route::Processes(ProcessesRoute::Overview));
        assert!(back.chain().published::<AgentStateRequest>(), "{:#?}", back.chain());
        assert!(listed(&mut app, "notepad.exe"), "kept rows show at once: {:#?}", app.tree());
        assert!(!listed(&mut app, "calc.exe"), "a report sent while asleep is not applied");

        h.advance(Duration::from_secs(1));
        app.settle();
        assert!(listed(&mut app, "calc.exe"), "{:#?}", app.tree());
    }
}
