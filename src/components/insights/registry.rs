use exo_types::insights::{
    InsightMeta, binary_systems, crowded_systems, distant_exoplanets,
    equal_star_planet_pairs, hottest_stellar_hosts, largest_exoplanets,
    nearest_stellar_hosts, planet_host_ratios, smallest_exoplanets,
};
use leptos::prelude::*;

use super::binary_systems::BinaryStarSystemsPage;
use super::crowded_systems::SystemsWithMostPlanetsPage;
use super::distant_exoplanets::MostDistantExoplanetsPage;
use super::equal_star_planet_pairs::MostEqualStarPlanetPairsPage;
use super::hottest_stellar_hosts::HottestStellarHostsPage;
use super::largest_exoplanets::LargestExoplanetsPage;
use super::nearest_stellar_hosts::NearestStellarHostsPage;
use super::planet_host_ratios::LargestPlanetToHostRatiosPage;
use super::smallest_exoplanets::SmallestExoplanetsPage;

pub struct InsightPage {
    pub meta: &'static InsightMeta,
    pub render: fn() -> AnyView,
}

pub static PAGES: &[InsightPage] = &[
    InsightPage {
        meta: &smallest_exoplanets::META,
        render: render_smallest_exoplanets,
    },
    InsightPage {
        meta: &largest_exoplanets::META,
        render: render_largest_exoplanets,
    },
    InsightPage {
        meta: &distant_exoplanets::META,
        render: render_distant_exoplanets,
    },
    InsightPage {
        meta: &nearest_stellar_hosts::META,
        render: render_nearest_stellar_hosts,
    },
    InsightPage {
        meta: &planet_host_ratios::META,
        render: render_planet_host_ratios,
    },
    InsightPage {
        meta: &equal_star_planet_pairs::META,
        render: render_equal_star_planet_pairs,
    },
    InsightPage {
        meta: &hottest_stellar_hosts::META,
        render: render_hottest_stellar_hosts,
    },
    InsightPage {
        meta: &crowded_systems::META,
        render: render_crowded_systems,
    },
    InsightPage {
        meta: &binary_systems::META,
        render: render_binary_systems,
    },
];

pub fn find_page(slug: &str) -> Option<&'static InsightPage> {
    PAGES.iter().find(|page| page.meta.slug == slug)
}

fn render_smallest_exoplanets() -> AnyView {
    view! { <SmallestExoplanetsPage/> }.into_any()
}

fn render_largest_exoplanets() -> AnyView {
    view! { <LargestExoplanetsPage/> }.into_any()
}

fn render_distant_exoplanets() -> AnyView {
    view! { <MostDistantExoplanetsPage/> }.into_any()
}

fn render_nearest_stellar_hosts() -> AnyView {
    view! { <NearestStellarHostsPage/> }.into_any()
}

fn render_planet_host_ratios() -> AnyView {
    view! { <LargestPlanetToHostRatiosPage/> }.into_any()
}

fn render_equal_star_planet_pairs() -> AnyView {
    view! { <MostEqualStarPlanetPairsPage/> }.into_any()
}

fn render_hottest_stellar_hosts() -> AnyView {
    view! { <HottestStellarHostsPage/> }.into_any()
}

fn render_crowded_systems() -> AnyView {
    view! { <SystemsWithMostPlanetsPage/> }.into_any()
}

fn render_binary_systems() -> AnyView {
    view! { <BinaryStarSystemsPage/> }.into_any()
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::PAGES;
    use exo_types::insights;

    #[test]
    fn page_registry_matches_insight_metadata_registry() {
        let page_slugs =
            PAGES.iter().map(|page| page.meta.slug).collect::<Vec<_>>();
        let meta_slugs = insights::INSIGHTS
            .iter()
            .map(|meta| meta.slug)
            .collect::<Vec<_>>();

        assert_eq!(page_slugs, meta_slugs);
    }
}
