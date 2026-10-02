use app_contracts::features::sidebar::SidebarState;
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

use super::actor::{Refresh, SidebarActor};
use super::settings::SidebarSettings;

feature! {
    pub SidebarFeature {
        exports { SidebarState }
    }
}

#[installs]
fn sidebar(cx: &FeatureInitContext) -> anyhow::Result<SidebarFeature> {
    let settings = cx.settings::<SidebarSettings>()?;

    let seed = SidebarState {
        open: settings.open().get(),
        width: settings.width().get(),
    };

    let (sidebar, _) = cx
        .state::<SidebarState>()
        .seed(seed)
        .driven_by(|push| SidebarActor::new(push, settings));

    sidebar.emit(Refresh);

    Ok(SidebarFeature(sidebar))
}
