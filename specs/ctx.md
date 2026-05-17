# Current Context: Sitemap chunking complete

## Goal

Split the generated exoplanet and stellar host sitemaps into smaller child
sitemaps, keep URL lists deduplicated, and use canonical path-segment URL
encoding.

## Completed

- `src/server/handlers.rs` now builds chunked entity sitemap files with a
  fixed 1,000 URL chunk size.
- `/sitemap-index.xml` and `/sitemap-static.xml` remain.
- Entity routes are now `/sitemap-exoplanets-1.xml`,
  `/sitemap-stellarhosts-1.xml`, etc.
- The old `/sitemap-exoplanets.xml` and `/sitemap-stellarhosts.xml` routes are
  not registered.
- Entity URL lists remain sorted and deduplicated before chunking.
- Detail path segments use sitemap-specific encoding that leaves unreserved
  characters readable and percent-encodes spaces and reserved/path-breaking
  characters.
- `specs/sitemap.md` has been updated to describe the chunked structure.

## Verification

- `cargo test test_sitemap` passed.
- `cargo test entity_sitemap` passed.
- `cargo fmt --check` passed.
- `cargo check` passed.
