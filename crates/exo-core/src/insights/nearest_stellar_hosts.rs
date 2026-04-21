use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::nearest_stellar_hosts::META,
    table: InsightTable::StellarHosts,
    sql: r#"
        SELECT hostname, MIN(sy_dist) AS sy_dist, MAX(st_teff) AS st_teff, MAX(st_mass) AS st_mass, MAX(sy_pnum) AS sy_pnum
        FROM stellarhosts
        WHERE hostname IS NOT NULL
          AND hostname != ''
          AND sy_dist IS NOT NULL
        GROUP BY hostname
        ORDER BY sy_dist ASC, hostname ASC
        LIMIT 10
    "#,
};
