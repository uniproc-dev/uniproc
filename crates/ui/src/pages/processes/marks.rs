#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessesMark {
    EndTask,
    Chevron,
    Exited,
}
