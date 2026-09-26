#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServicesMark {
    Start,
    Stop,
    Restart,
}
