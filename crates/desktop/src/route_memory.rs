use amethystate::amethystate;
use app_contracts::features::settings::StartPage;
use domain::features::settings::settings::GeneralSettings;
use amethystate::Store;
use guinea_plugin_store::StoreAccess;

use crate::routes::{Route, ProcessesRoute, ActivityRoute};

#[amethystate(prefix = "shell")]
pub struct RouteSettings {
    #[amestate(default = String::new())]
    last_route: String,
}

pub fn open(store: &Store) -> anyhow::Result<RouteSettings> {
    Ok(store.try_settings::<RouteSettings>()?)
}

pub fn remember(settings: &RouteSettings, route: &Route) {
    if matches!(
        route,
        Route::Settings {}
            | Route::Processes(ProcessesRoute::Settings)
            | Route::Activity(ActivityRoute::Groups)
            | Route::System(_)
    ) {
        return;
    }
    let Some(saved) = route.save() else {
        return;
    };
    settings.last_route().set(saved);
}

pub fn restore(store: &Store) -> anyhow::Result<Route> {
    let page = store.try_settings::<GeneralSettings>()?.start_page_choice();
    let last = if page == StartPage::LastOpened {
        Route::restore(&open(store)?.last_route().get())
    } else {
        None
    };
    Ok(route_for(page, || last))
}

fn route_for(page: StartPage, last: impl FnOnce() -> Option<Route>) -> Route {
    match page {
        StartPage::LastOpened => last().unwrap_or(Route::Processes(ProcessesRoute::Overview)),
        StartPage::Processes => Route::Processes(ProcessesRoute::Overview),
        StartPage::Services => Route::Services {},
        StartPage::Wsl => Route::Wsl {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[guinea::test(iterations = 1)]
    fn unreadable_route_settings_return_an_error(h: &mut guinea::app::Harness) {
        let store = crate::test_support::install(h);
        store.set(["shell", "last_route"], &42u64).unwrap();

        assert!(open(&store).is_err());
        store.try_settings::<GeneralSettings>().unwrap().start_page().set(StartPage::LastOpened.id().to_owned());
        assert!(restore(&store).is_err());
    }

    #[test]
    fn a_chosen_start_page_wins_over_the_last_one() {
        let last = || Some(Route::Wsl {});
        assert!(matches!(route_for(StartPage::Services, last), Route::Services {}));
        assert!(matches!(route_for(StartPage::LastOpened, last), Route::Wsl {}));
        assert!(matches!(route_for(StartPage::LastOpened, || None), Route::Processes(ProcessesRoute::Overview)));
    }
}
