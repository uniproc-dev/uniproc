use rpstate::rpstate;

#[rpstate(prefix = "navigation")]
pub struct NavigationSettings {
    #[state(default = "processes".to_string())]
    pub default_route_segment: String,
}
