use super::{InsightDef, InsightTable};

pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::hottest_stellar_hosts::META,
    table: InsightTable::StellarHosts,
    sql: r#"
        SELECT hostname, MAX(st_teff) AS st_teff, MAX(st_mass) AS st_mass, MIN(sy_dist) AS sy_dist, MAX(sy_pnum) AS sy_pnum
        FROM stellarhosts
        WHERE hostname IS NOT NULL
          AND hostname != ''
          AND st_teff IS NOT NULL
        GROUP BY hostname
        ORDER BY st_teff DESC, hostname ASC
        LIMIT 10
    "#,
};
