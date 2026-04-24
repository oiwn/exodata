use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::crowded_systems::META,
    table: InsightTable::StellarHosts,
    sql: r#"
        SELECT sy_name, MIN(hostname) AS host_link_hostname, MAX(sy_pnum) AS sy_pnum, MAX(sy_snum) AS sy_snum, MIN(sy_dist) AS sy_dist
        FROM stellarhosts
        WHERE sy_name IS NOT NULL
          AND sy_name != ''
          AND sy_pnum IS NOT NULL
        GROUP BY sy_name
        ORDER BY sy_pnum DESC, sy_name ASC
        LIMIT 10
    "#,
};
