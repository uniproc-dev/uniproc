#[derive(Clone)]
pub struct SortState<SID> {
    pub field_id: Option<SID>,
    pub descending: bool,
}
