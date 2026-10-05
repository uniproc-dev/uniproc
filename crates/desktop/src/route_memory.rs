use amethystate::{amethystate, Open};
use app_contracts::features::settings::StartPage;
use app_contracts::OrWarn;
use domain::features::settings::settings::GeneralSettings;
use guinea::feature::{FeatureInitContext, ScopeContext};
use guinea_plugin_store::StoreAccess;

use crate::routes::Route;

#[amethystate(prefix = "shell")]
pub struct RouteSettings {
    #[amestate(default = String::new())]
    last_route: String,
}

pub fn open(ctx: &FeatureInitContext) -> Option<RouteSettings> {
    ctx.settings::<RouteSettings>().or_warn("could not open the route settings")
}

pub fn remember(settings: &RouteSettings, route: &Route) {
    if matches!(
        route,
        Route::Settings {}
            | Route::ProcessesSettings {}
            | Route::ActivityGroups {}
            | Route::System {}
            | Route::SystemTools {}
    ) {
        return;
    }
    let Some(saved) = route.save() else {
        return;
    };
    settings.last_route().set(saved).or_warn("could not remember the current route");
}

fn opened<S: Open>(cx: &ScopeContext) -> Option<S> {
    cx.settings::<S>().or_warn(format_args!("could not open {}", std::any::type_name::<S>()))
}

pub fn restore(cx: &ScopeContext) -> Route {
    let page = opened::<GeneralSettings>(cx).map(|general| general.start_page_choice()).unwrap_or_default();
    route_for(page, || {
        opened::<RouteSettings>(cx).and_then(|settings| Route::restore(&settings.last_route().get()))
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
