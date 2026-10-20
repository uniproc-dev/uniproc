#[derive(Clone, PartialEq, Debug, guinea::Routable)]
pub enum Route {
    Processes {},
    ProcessesSettings {},
    Activity {},
    ActivityGroups {},
    Services {},
    Wsl {},
    System {},
    SystemTools {},
    Settings {},
}

impl Route {
    pub fn save(&self) -> Option<String> {
        Some(match self {
            Self::Processes {} => "Processes",
            Self::ProcessesSettings {} => "ProcessesSettings",
            Self::Activity {} => "Activity",
            Self::ActivityGroups {} => "ActivityGroups",
            Self::Services {} => "Services",
            Self::Wsl {} => "Wsl",
            Self::System {} => "System",
            Self::SystemTools {} => "SystemTools",
            Self::Settings {} => "Settings",
        }.to_owned())
    }

    pub fn restore(saved: &str) -> Option<Self> {
        Some(match saved {
            "Processes" => Self::Processes {},
            "ProcessesSettings" => Self::ProcessesSettings {},
            "Activity" => Self::Activity {},
            "ActivityGroups" => Self::ActivityGroups {},
            "Services" => Self::Services {},
            "Wsl" => Self::Wsl {},
            "System" => Self::System {},
            "SystemTools" => Self::SystemTools {},
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
            Route::Processes {},
            Route::ProcessesSettings {},
            Route::Activity {},
            Route::ActivityGroups {},
            Route::Services {},
            Route::Wsl {},
            Route::System {},
            Route::SystemTools {},
            Route::Settings {},
        ] {
            assert_eq!(Route::restore(&route.save().unwrap()), Some(route));
        }
        assert_eq!(Route::restore(""), None);
        assert_eq!(Route::restore("unknown"), None);
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
        h.provide(ProcessesDeps {
            windows: AppWindows::default,
            shell: |_| {},
        });
        h.app(crate::app::with_fakes).unwrap();
        h.provide(test_agent::agent_link());
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
        let mut app = crate::test_support::mount_application(h, Route::Activity {}).unwrap();
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
        let mut app = crate::test_support::mount_application(h, Route::Processes {}).unwrap();
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
