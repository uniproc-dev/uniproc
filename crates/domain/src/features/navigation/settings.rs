use amethystate::amethystate;

#[amethystate(prefix = "navigation")]
pub struct NavigationSettings {
    #[amestate(default = "processes".to_string())]
    pub default_route_segment: String,
}
