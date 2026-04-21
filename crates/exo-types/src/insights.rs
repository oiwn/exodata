#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsightMeta {
    pub slug: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub limit: usize,
}

pub mod smallest_exoplanets {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "smallest-exoplanets-radius",
        title: "Smallest Exoplanets By Radius",
        category: "Planetary extremes",
        description: "Tiny confirmed worlds ordered by radius with host-star context.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod largest_exoplanets {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "largest-exoplanets-radius",
        title: "Largest Exoplanets By Radius",
        category: "Planetary extremes",
        description: "Inflated giants and outsized worlds ranked by radius with quick host-star context.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod distant_exoplanets {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "most-distant-exoplanets",
        title: "Most Distant Exoplanets From Their Stars",
        category: "Orbital architecture",
        description: "Confirmed planets ranked by semi-major axis instead of inferred temperature.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod nearest_stellar_hosts {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "nearest-stellar-hosts",
        title: "Nearest Stellar Hosts",
        category: "Distance",
        description: "Nearby host stars ranked by distance from Earth for quick local-neighborhood browsing.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod planet_host_ratios {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "largest-planet-to-host-ratios",
        title: "Largest Planet-To-Host Ratios",
        category: "Relationships",
        description: "Extreme size-ratio systems highlighting oversized planets around comparatively small stars.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod equal_star_planet_pairs {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "most-equal-star-planet-pairs",
        title: "Most Equal Star-Planet Pairs",
        category: "Relationships",
        description: "Systems where planet and host-star sizes sit unusually close together in relative scale.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod hottest_stellar_hosts {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "hottest-stellar-hosts",
        title: "Hottest Stellar Hosts",
        category: "Stellar extremes",
        description: "Host stars with the highest effective temperatures among confirmed systems.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod crowded_systems {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "systems-with-most-planets",
        title: "Planetary Systems With Most Planets",
        category: "System architecture",
        description: "System names ordered by the archive-backed confirmed planet count.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub mod binary_systems {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "binary-star-systems",
        title: "Binary Planetary Systems With Planets",
        category: "System architecture",
        description: "Planetary systems where the archive star count identifies two stars.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub static INSIGHTS: &[&InsightMeta] = &[
    &smallest_exoplanets::META,
    &largest_exoplanets::META,
    &distant_exoplanets::META,
    &nearest_stellar_hosts::META,
    &planet_host_ratios::META,
    &equal_star_planet_pairs::META,
    &hottest_stellar_hosts::META,
    &crowded_systems::META,
    &binary_systems::META,
];

pub fn find_insight(slug: &str) -> Option<&'static InsightMeta> {
    INSIGHTS.iter().copied().find(|meta| meta.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::INSIGHTS;

    #[test]
    fn insight_slugs_are_unique() {
        for (index, insight) in INSIGHTS.iter().enumerate() {
            assert!(
                !INSIGHTS[..index]
                    .iter()
                    .any(|previous| previous.slug == insight.slug),
                "duplicate insight slug: {}",
                insight.slug
            );
        }
    }
}
