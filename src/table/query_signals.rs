use leptos::prelude::*;

use crate::table::TableQueryState;

#[derive(Clone, Copy)]
pub struct TableQuerySignals {
    pub current_page: ReadSignal<usize>,
    pub set_current_page: WriteSignal<usize>,
    pub sort_column: ReadSignal<Option<String>>,
    pub set_sort_column: WriteSignal<Option<String>>,
    pub sort_order: ReadSignal<String>,
    pub set_sort_order: WriteSignal<String>,
    pub selected_columns: ReadSignal<Vec<String>>,
    pub set_selected_columns: WriteSignal<Vec<String>>,
    pub filter_text: ReadSignal<String>,
    pub set_filter_text: WriteSignal<String>,
    pub filter_input: ReadSignal<String>,
    pub set_filter_input: WriteSignal<String>,
}

impl TableQuerySignals {
    pub fn new(
        initial_page: usize,
        initial_sort_column: Option<String>,
        initial_sort_order: String,
        initial_columns: Vec<String>,
        initial_filter: String,
    ) -> Self {
        let (current_page, set_current_page) = signal(initial_page);
        let (sort_column, set_sort_column) = signal(initial_sort_column);
        let (sort_order, set_sort_order) = signal(initial_sort_order);
        let (selected_columns, set_selected_columns) = signal(initial_columns);
        let (filter_text, set_filter_text) = signal(initial_filter.clone());
        let (filter_input, set_filter_input) = signal(initial_filter);

        Self {
            current_page,
            set_current_page,
            sort_column,
            set_sort_column,
            sort_order,
            set_sort_order,
            selected_columns,
            set_selected_columns,
            filter_text,
            set_filter_text,
            filter_input,
            set_filter_input,
        }
    }

    pub fn query(&self) -> TableQueryState {
        TableQueryState::new(
            self.current_page.get(),
            self.sort_column.get(),
            self.sort_order.get(),
            self.selected_columns.get(),
            self.filter_text.get(),
        )
    }

    pub fn query_with_page(&self, page: usize) -> TableQueryState {
        TableQueryState::new(
            page,
            self.sort_column.get(),
            self.sort_order.get(),
            self.selected_columns.get(),
            self.filter_text.get(),
        )
    }

    pub fn query_with_sort(
        &self,
        page: usize,
        sort_column: Option<String>,
        sort_order: String,
    ) -> TableQueryState {
        TableQueryState::new(
            page,
            sort_column,
            sort_order,
            self.selected_columns.get(),
            self.filter_text.get(),
        )
    }

    pub fn query_with_columns(
        &self,
        page: usize,
        sort_column: Option<String>,
        columns: Vec<String>,
    ) -> TableQueryState {
        TableQueryState::new(
            page,
            sort_column,
            self.sort_order.get(),
            columns,
            self.filter_text.get(),
        )
    }

    pub fn query_with_filter(
        &self,
        page: usize,
        filter: String,
    ) -> TableQueryState {
        TableQueryState::new(
            page,
            self.sort_column.get(),
            self.sort_order.get(),
            self.selected_columns.get(),
            filter,
        )
    }
}
