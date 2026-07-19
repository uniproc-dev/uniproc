use forsl_macros::bindings;

#[bindings]
pub trait UiNavigationBindings: 'static {
    #[manual]
    #[tracing(target = "path")]
    fn on_push<F>(&self, handler: F)
    where
        F: Fn(String) + 'static;
}
