use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::smallest_exoplanets::META,
    table: InsightTable::Exoplanets,
    sql: r#"
        SELECT pl_name, hostname, pl_rade, pl_bmasse, disc_year
        FROM exoplanets
        WHERE default_flag = 1
          AND pl_name IS NOT NULL
          AND pl_name != ''
          AND pl_rade IS NOT NULL
        ORDER BY pl_rade ASC, pl_name ASC
        LIMIT 10
    "#,
};
