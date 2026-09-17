# Exoplanet Detail Page

The route `/exoplanets/:pl_name` renders all matching source records together
with a canonical summary, deterministic CSS visual, scale comparison, and
provenance exports. Locale aliases reuse the same component.

## Implementation

The parent module is [exoplanet_detail.rs](../src/components/exoplanet_detail.rs).
Its children are `page.rs`, `hero.rs`, `comparison.rs`, `summary.rs`,
`records.rs`, and `format.rs`. Feature styles live in
[exoplanet-detail.css](../style/components/exoplanet-detail.css), imported by
`style/tailwind.css`. Follow the shared
[styling convention](web-frontend.md#styling-with-tailwind-css).

The page renders a back link, hero, canonical summary, radius comparison, and
provenance section. It loads through a lazy route and a Leptos resource.
Loading and resource errors use feature-specific views; missing entities do
not yet use the shared branded 404.

## Data Contract

`get_exoplanet_detail` returns the shared type from
[functions.rs](../src/server/functions.rs):

```rust
pub struct ExoplanetDetail {
    pub selected_record_index: Option<usize>,
    pub pl_name: String,
    pub canonical: ExoplanetCanonicalSummary,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}
```

Exact planet-name matching returns all source rows. The canonical builder in
[exoplanet_canonical.rs](../src/server/exoplanet_canonical.rs) reuses the host
record-summary helpers.

| Canonical field | Source and summary |
| --- | --- |
| `hostname` | Stable `hostname` summary |
| `discovery_method` | Most frequent `discoverymethod` |
| `discovery_year` | Stable `disc_year` summary |
| `orbital_period` | Numeric `pl_orbper` |
| `semi_major_axis` | Numeric `pl_orbsmax` |
| `radius` | Numeric `pl_rade` |
| `mass` | Numeric `pl_bmasse`, falling back to `pl_masse` only if no usable best-mass summary exists |
| `density` | Numeric `pl_dens` |
| `equilibrium_temperature` | Numeric `pl_eqt` |

Shared `exo-core` selection chooses the unique `default_flag = 1` row.
Missing or ambiguous defaults yield no adopted values. `selected_record_index`
points into the unchanged raw `records` array. All primary numeric, stable, and
categorical values come from this row, without medians or cross-row filling.
The mass fallback to `pl_masse` is restricted to this same row. Numeric counts,
distinct counts, ranges, and categorical alternatives describe all records.
Missing selected fields produce no summary. Disagreement means distinct source values,
not statistical significance. See
[backend details](web-backend.md#detail-contracts) for shared summary contracts.

## Current Rendering

- **Hero:** name, host link, record count, discovery information, and selected
  radius/mass values from the canonical summary. Missing stats use placeholders.
- **Visual:** local CSS classes selected deterministically from selected radius
  and equilibrium temperature. Selection order is hot, giant, sub-neptune,
  cold, then temperate. It is a decorative classification, not a measured image.
- **Summary:** renders canonical numeric, stable, and categorical values with
  counts and disagreement context.
- **Comparison:** uses the canonical selected radius and
  compares linear radii with Earth (1) and Jupiter (11.2). Display diameters
  have a minimum size. No radius means no visible comparison section.
- **Provenance:** summary metrics, references, and a source-record table.
  JSON exports return the full detail payload; CSV exports return matching
  source rows. Links use encoded names under the unprefixed detail route.

The current payload has no separate `identity`, `visual`, or `provenance`
fields. Proposed extensions and changes to summary consumption live in
[ideas.md](ideas.md#detail-page-follow-ups).

## Verification

Canonical-summary and formatting tests cover source selection, missing fields, mass
fallback, classification, and scaling. Shared export tests cover serialization
and filenames; browser smoke tests cover mobile provenance containment.
See [testing.md](testing.md) and the
[checks skill](../.agents/skills/exodata-checks/SKILL.md).
