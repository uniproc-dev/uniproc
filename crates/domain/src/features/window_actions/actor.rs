use app_contracts::features::window_actions::{
    ResizeEdge, UiWindowActionsBindings, UiWindowActionsPort, WindowActionsBinder,
    WindowActionsPartialBinder, WindowBreakpoint, WindowConfigChanged,
};
use forsl_core::actor::{Context, ManagedActor};
use macros::{actor_manifest, handler};

#[actor_manifest(binder = WindowActionsBinder)]
impl<P: UiWindowActionsPort> ManagedActor for WindowActor<P> {
    type Bus = bus!();
    type Handlers = handlers!(
        bind {
            Drag,
            Close,
            Minimize,
            Maximize,
            StartResize(ResizeEdge),
            ConfigChanged(WindowBreakpoint, u64)
        },
    );
    type Signals = bus!(WindowConfigChanged);
}

pub struct WindowActor<P> {
    pub port: P,
}

#[handler]
fn drag_window<P: UiWindowActionsPort>(this: &mut WindowActor<P>, _: Drag) {
    this.port.drag_window();
}

#[handler]
fn close_window<P: UiWindowActionsPort>(this: &mut WindowActor<P>, _: Close) {
    this.port.close_window();
}

#[handler]
fn minimize_window<P: UiWindowActionsPort>(this: &mut WindowActor<P>, _: Minimize) {
    this.port.minimize_window();
}

#[handler]
fn maximize_window<P: UiWindowActionsPort>(this: &mut WindowActor<P>, _: Maximize) {
    this.port.toggle_maximize_window();
}

#[handler]
fn resize_window<P: UiWindowActionsPort>(this: &mut WindowActor<P>, msg: StartResize) {
    this.port.resize_window(msg.0);
}

#[handler]
fn on_breakpoint_changed<P: UiWindowActionsPort>(
    _: &mut WindowActor<P>,
    msg: ConfigChanged,
    ctx: &Context<WindowActor<P>>,
) {
    ctx.publish(WindowConfigChanged {
        breakpoint: msg.0,
        width: msg.1,
    });
}
