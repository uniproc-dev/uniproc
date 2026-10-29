#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServicesMark {
    RunNewTask,
    OpenServices,
    OpenSettings,
    SelectedIndicator,
    SortIndicator,
    Menu,
    MenuBackdrop,
    MenuMoreColumns,
    Start,
    Stop,
    Restart,
    Operations,
    OperationsToggle,
    OperationsScroll,
    Operation,
    OperationIndicator,
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
