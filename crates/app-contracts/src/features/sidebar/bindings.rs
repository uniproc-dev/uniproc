use forsl_macros::bindings;

#[bindings]
pub trait UiSidebarBindings: 'static {
    #[slint(arg_types = "length")]
    fn on_side_bar_width_changed<F>(&self, handler: F)
    where
        F: Fn(u64) + 'static;
}
