use framework::app::Window;
mod actor;

use crate::features::window_actions::actor::WindowActor;
use app_contracts::features::window_actions::{
    UiWindowActionsBindings, UiWindowActionsPort,
};
use framework::addr::AddrBuilder;
use framework::feature::{WindowFeature, WindowFeatureInitContext};
use macros::window_feature;

#[window_feature]
pub struct WindowActionsFeature;

#[window_feature]
impl<TWindow, F, P> WindowFeature<TWindow> for WindowActionsFeature<F>
where
    TWindow: Window,
    F: Fn(&TWindow) -> P + 'static + Clone,
    P: UiWindowActionsPort + UiWindowActionsBindings,
{
    fn install(&mut self, ctx: &mut WindowFeatureInitContext<TWindow>) -> anyhow::Result<()> {
        let port = (self.make_port)(ctx.ui);
        let token = ctx.ui.new_token();

        AddrBuilder::new(token, &self.tracker)
            .managed(WindowActor { port: port.clone() })
            .ui_bind(&port);

        Ok(())
    }
}
