use std::sync::OnceLock;

use amethystate::amethystate;
use app_contracts::features::settings::StartPage;
use domain::features::settings::settings::GeneralSettings;

use crate::routes::Route;

#[amethystate(prefix = "shell")]
pub struct RouteSettings {
    #[amestate(default = String::new())]
    last_route: String,
}

static SETTINGS: OnceLock<Option<RouteSettings>> = OnceLock::new();

fn settings() -> Option<&'static RouteSettings> {
    SETTINGS
        .get_or_init(|| match RouteSettings::new() {
            Ok(settings) => Some(settings),
            Err(err) => {
                tracing::warn!(?err, "could not open the route settings");
                None
            }
        })
        .as_ref()
}

pub fn remember(route: &Route) {
    if matches!(route, Route::Settings {}) {
        return;
    }
    let (Some(settings), Some(saved)) = (settings(), route.save()) else {
        return;
    };
    if let Err(err) = settings.last_route().set(saved) {
        tracing::warn!(?err, "could not remember the current route");
    }
}

fn start_page() -> StartPage {
    match GeneralSettings::new() {
        Ok(general) => general.start_page_choice(),
        Err(err) => {
            tracing::warn!(?err, "could not open the general settings");
            StartPage::default()
        }
    }
}

pub fn restore() -> Route {
    route_for(start_page(), || {
        settings().and_then(|settings| Route::restore(&settings.last_route().get()))
    })
}

fn route_for(page: StartPage, last: impl FnOnce() -> Option<Route>) -> Route {
    match page {
        StartPage::LastOpened => last().unwrap_or(Route::Processes {}),
        StartPage::Processes => Route::Processes {},
        StartPage::Services => Route::Services {},
        StartPage::Wsl => Route::Wsl {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_start_page_wins_over_the_last_one() {
        let last = || Some(Route::Wsl {});
        assert!(matches!(route_for(StartPage::Services, last), Route::Services {}));
        assert!(matches!(route_for(StartPage::LastOpened, last), Route::Wsl {}));
        assert!(matches!(route_for(StartPage::LastOpened, || None), Route::Processes {}));
    }
}
