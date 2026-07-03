use rpstate::rpstate;

#[rpstate(prefix = "sidebar")]
pub struct SidebarSettings {
    #[state(default = 120u64)]
    pub switch_hide_delay_ms: u64,

    #[state(default = 40u64)]
    pub switch_show_delay_ms: u64,

    #[state(default = 260u64)]
    pub width: u64,
}
