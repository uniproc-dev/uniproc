use std::sync::OnceLock;

use amethystate::amethystate;

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
    let (Some(settings), Some(saved)) = (settings(), route.save()) else {
        return;
    };
    if let Err(err) = settings.last_route().set(saved) {
        tracing::warn!(?err, "could not remember the current route");
    }
}

pub fn restore() -> Route {
    settings()
        .and_then(|settings| Route::restore(&settings.last_route().get()))
        .unwrap_or(Route::Processes {})
}
