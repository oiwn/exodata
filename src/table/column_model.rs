use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct ColumnGroup {
    pub base: String,
    pub err1: Option<String>,
    pub err2: Option<String>,
    pub lim: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ColumnModel {
    /// Base columns only (used for headers/selector).
    pub display_columns: Vec<String>,
    /// Columns to fetch from server (base + err/lim companions).
    pub fetch_columns: Vec<String>,
    /// Base -> companion column names.
    pub groups: HashMap<String, ColumnGroup>,
    /// All err/lim columns (for filtering in selector).
    pub hidden_columns: HashSet<String>,
}

pub fn is_err_or_lim(name: &str) -> bool {
    name.ends_with("err1") || name.ends_with("err2") || name.ends_with("lim")
}

fn base_and_kind(name: &str) -> Option<(&str, &str)> {
    if let Some(base) = name.strip_suffix("err1") {
        return Some((base, "err1"));
    }
    if let Some(base) = name.strip_suffix("err2") {
        return Some((base, "err2"));
    }
    if let Some(base) = name.strip_suffix("lim") {
        return Some((base, "lim"));
    }
    None
}

/// Build a light column model from metadata keys and selected base columns.
/// Note: this stores column names only (no row data).
pub fn build_column_model(
    all_columns: &[String],
    selected_columns: &[String],
) -> ColumnModel {
    let mut groups: HashMap<String, ColumnGroup> = HashMap::new();
    let mut hidden_columns: HashSet<String> = HashSet::new();

    for col in all_columns {
        if let Some((base, kind)) = base_and_kind(col) {
            hidden_columns.insert(col.clone());
            let entry = groups.entry(base.to_string()).or_insert(ColumnGroup {
                base: base.to_string(),
                err1: None,
                err2: None,
                lim: None,
            });
            match kind {
                "err1" => entry.err1 = Some(col.clone()),
                "err2" => entry.err2 = Some(col.clone()),
                "lim" => entry.lim = Some(col.clone()),
                _ => {}
            }
        }
    }

    let display_columns: Vec<String> = selected_columns
        .iter()
        .filter(|c| !hidden_columns.contains(*c))
        .cloned()
        .collect();

    let mut fetch_columns = display_columns.clone();
    for base in &display_columns {
        if let Some(group) = groups.get(base) {
            for col in
                [&group.err1, &group.err2, &group.lim].into_iter().flatten()
            {
                if !fetch_columns.contains(col) {
                    fetch_columns.push(col.clone());
                }
            }
        }
    }

    ColumnModel {
        display_columns,
        fetch_columns,
        groups,
        hidden_columns,
    }
}
