use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::equal_star_planet_pairs::META,
    table: InsightTable::Exoplanets,
    sql: r#"
        SELECT
            pl_name,
            hostname,
            pl_rade / (st_rad * 109.076) AS pl_host_radius_ratio,
            ABS((pl_rade / (st_rad * 109.076)) - 1.0) AS pl_host_radius_ratio_delta,
            pl_rade,
            st_rad,
            disc_year
        FROM exoplanets
        WHERE default_flag = 1
          AND pl_name IS NOT NULL
          AND pl_name != ''
          AND pl_rade IS NOT NULL
          AND pl_rade > 0
          AND st_rad IS NOT NULL
          AND st_rad > 0
        ORDER BY pl_host_radius_ratio_delta ASC, pl_name ASC
        LIMIT 10
    "#,
};
