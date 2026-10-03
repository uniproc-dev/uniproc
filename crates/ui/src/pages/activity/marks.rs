#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivityMark {
    Came,
    Went,
    NewOnly,
    Bursts,
    Search,
    ClearRange,
    Histogram,
    Rows,
    Facts,
    Empty,
}
