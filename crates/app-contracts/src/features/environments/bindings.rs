use forsl_macros::bindings;

#[bindings]
pub trait UiEnvironmentsBindings: 'static {
    #[tracing(target = "agent")]
    fn on_install_agent<F>(&self, handler: F)
    where
        F: Fn(String) + 'static;
}
