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

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn detects_error_and_limit_companion_columns() {
        assert!(is_err_or_lim("pl_radeerr1"));
        assert!(is_err_or_lim("pl_radeerr2"));
        assert!(is_err_or_lim("pl_radelim"));
        assert!(!is_err_or_lim("pl_rade"));
        assert!(!is_err_or_lim("pl_rade_error"));
    }

    #[test]
    fn builds_display_and_fetch_columns_with_companions() {
        let model = build_column_model(
            &strings(&[
                "pl_name",
                "pl_rade",
                "pl_radeerr1",
                "pl_radeerr2",
                "pl_radelim",
                "hostname",
            ]),
            &strings(&["pl_name", "pl_rade", "pl_radeerr1", "hostname"]),
        );

        assert_eq!(
            model.display_columns,
            strings(&["pl_name", "pl_rade", "hostname"])
        );
        assert_eq!(
            model.fetch_columns,
            strings(&[
                "pl_name",
                "pl_rade",
                "hostname",
                "pl_radeerr1",
                "pl_radeerr2",
                "pl_radelim",
            ])
        );
        assert!(model.hidden_columns.contains("pl_radeerr1"));
        assert_eq!(model.groups["pl_rade"].base, "pl_rade");
        assert_eq!(model.groups["pl_rade"].err1.as_deref(), Some("pl_radeerr1"));
        assert_eq!(model.groups["pl_rade"].err2.as_deref(), Some("pl_radeerr2"));
        assert_eq!(model.groups["pl_rade"].lim.as_deref(), Some("pl_radelim"));
    }
}
