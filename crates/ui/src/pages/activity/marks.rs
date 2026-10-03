#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivityMark {
    Came,
    Went,
    NewOnly,
    Bursts,
    Search,
    ClearArea,
    Scatter,
    Card,
    Rows,
    Row,
    Menu,
    Facts,
    Empty,
}
