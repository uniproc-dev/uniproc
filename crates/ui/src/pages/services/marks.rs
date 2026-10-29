#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServicesMark {
    RunNewTask,
    OpenServices,
    Start,
    Stop,
    Restart,
}
