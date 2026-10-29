use amethystate::{ReactiveMap, amethystate};
use app_contracts::features::services::ServiceColumnConfig;

#[amethystate(prefix = "services")]
pub struct ServicesSettings {
    #[amestate(default = {})]
    columns: ReactiveMap<String, ServiceColumnConfig>,

    #[amestate(default = {})]
    order: ReactiveMap<String, u32>,
}
