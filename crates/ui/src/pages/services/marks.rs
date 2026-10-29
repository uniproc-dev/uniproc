#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServicesMark {
    RunNewTask,
    OpenServices,
    OpenSettings,
    SelectedIndicator,
    Start,
    Stop,
    Restart,
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServicesSettingsMark {
    Back,
    ColumnsGroup,
    Shown,
    Up,
    Down,
    ResetColumns,
}
