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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_preserves_all_fields() {
        let state = TablePaginationState::new(11, 20, 95, 2, 10, true, true);

        assert_eq!(state.start, 11);
        assert_eq!(state.end, 20);
        assert_eq!(state.total, 95);
        assert_eq!(state.current_page, 2);
        assert_eq!(state.total_pages, 10);
        assert!(state.can_go_prev);
        assert!(state.can_go_next);
    }
}
