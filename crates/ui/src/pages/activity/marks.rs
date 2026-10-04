#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivityMark {
    Came,
    Went,
    NewOnly,
    Series,
    Only,
    HideExe,
    HideFolder,
    HideLauncher,
    Picked,
    Search,
    ClearArea,
    Scatter,
    Card,
    Rows,
    Row,
    Selected,
    Menu,
    Preset,
    ManagePresets,
    RowMenu,
    RowMenuBackdrop,
    Facts,
    Empty,
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivityPresetsMark {
    Back,
    Name,
    Save,
    Apply,
    Current,
    Delete,
    Hidden,
    Empty,
}
