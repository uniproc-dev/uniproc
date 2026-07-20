mod apply;

use app_contracts::features::l10n::L10nPort;
use forsl::app::Window;
use forsl::feature::{WindowFeature, WindowFeatureInitContext};
use forsl_macros::window_feature;

#[window_feature]
pub fn l10n_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: L10nPort + Clone + 'static,
{
    apply::apply(&port);
    Ok(())
}
