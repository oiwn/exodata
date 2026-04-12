#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TablePaginationState {
    pub start: usize,
    pub end: usize,
    pub total: usize,
    pub current_page: usize,
    pub total_pages: usize,
    pub can_go_prev: bool,
    pub can_go_next: bool,
}

impl TablePaginationState {
    pub fn new(
        start: usize,
        end: usize,
        total: usize,
        current_page: usize,
        total_pages: usize,
        can_go_prev: bool,
        can_go_next: bool,
    ) -> Self {
        Self {
            start,
            end,
            total,
            current_page,
            total_pages,
            can_go_prev,
            can_go_next,
        }
    }
}
