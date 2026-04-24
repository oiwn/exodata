use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::distant_exoplanets::META,
    table: InsightTable::Exoplanets,
    sql: r#"
        SELECT pl_name, hostname, pl_orbsmax, pl_orbper, disc_year
        FROM exoplanets
        WHERE default_flag = 1
          AND pl_name IS NOT NULL
          AND pl_name != ''
          AND pl_orbsmax IS NOT NULL
        ORDER BY pl_orbsmax DESC, pl_name ASC
        LIMIT 10
    "#,
};
