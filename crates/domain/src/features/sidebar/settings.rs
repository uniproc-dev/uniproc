use amethystate::amethystate;

#[amethystate(prefix = "sidebar")]
pub struct SidebarSettings {
    #[amestate(default = 120u64)]
    pub switch_hide_delay_ms: u64,

    #[amestate(default = 40u64)]
    pub switch_show_delay_ms: u64,

    #[amestate(default = 260u64)]
    pub width: u64,
}
