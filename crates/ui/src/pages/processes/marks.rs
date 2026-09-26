#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessesMark {
    EndTask,
    GroupByType,
    Chevron,
    Exited,
    Menu,
    MenuBackdrop,
    MenuEndTask,
    MenuSuspend,
    MenuResume,
    MenuOpenFileLocation,
    MenuProperties,
    MenuSearchOnline,
    MenuSwitchTo,
    MenuMinimize,
    MenuMaximize,
    MenuCloseWindow,
}
