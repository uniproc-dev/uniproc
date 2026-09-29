use app_contracts::features::system::SystemTool;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToolMark(pub SystemTool);

impl guinea::Mark for ToolMark {
    fn name(&self) -> &'static str {
        self.0.id()
    }
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SystemMark {
    Open,
    Pin,
    Tools,
    Favourites,
    NoFavourites,
    Back,
}
