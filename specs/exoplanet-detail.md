# Exoplanet Detail Page Specification

## Purpose

Define the target architecture, content structure, and data contract for the exoplanet detail page.

This spec exists so the exoplanet detail refactor is driven by an explicit page model rather than by ad hoc parity with `stellarhost_detail`.

## Status

- Current implementation: `src/components/exoplanet_detail.rs` is a single-file component.
- Current backend payload: `ExoplanetDetail { pl_name, records, metadata }`
- This spec defines the target state for the next refactor phases.

## **TODO ASAP**

- Replace the thin exoplanet detail payload with a richer canonical summary/provenance contract.
- Move exoplanet summary cards from first-row/client-derived values to backend-produced adopted values and disagreement stats.
- Decide whether the exoplanet provenance section should gain working export actions or remain absent until a real export path exists.
- Treat this backend/data-contract follow-up as the highest-priority remaining gap in the exoplanet detail work.

## Design Intent

The exoplanet detail page must feel like the same product family as `stellarhost_detail`, while remaining planet-specific in content and structure.

Required alignment with `stellarhost_detail`:
- semantic feature module structure
- semantic feature CSS instead of large inline Tailwind blocks
- same page shell quality level
- similar loading and error treatment
- similar section rhythm and visual hierarchy

Required exoplanet-specific content:
- generated hero planet visual
- explicit scale comparison against Earth and Jupiter
- canonical/adopted planet summary from all rows
- access to full source records and provenance

## Non-Goals

- No forced one-to-one section parity with `stellarhost_detail`
- No scientific simulation of real planet appearance
- No freeform image generation service dependency
- No replacement of the raw records section with summary-only content

## Target File Structure

`src/components/exoplanet_detail/`

- `mod.rs`
- `page.rs`
- `hero.rs`
- `comparison.rs`
- `summary.rs`
- `records.rs`
- `format.rs`
- optional small helper modules when justified

`style/components/exoplanet-detail.css`

The public route export must remain stable for `app.rs`.

## Page Structure

The page should render in this order:

1. page shell
2. back link
3. hero section
4. canonical summary section
5. scale comparison section
6. records / provenance section

The page may add a small secondary section later, but v1 should avoid feature sprawl.

## Section Specifications

### 1. Hero Section

Purpose:
- establish identity of the planet
- present the strongest high-signal facts immediately
- provide a generated planet visual analogous to the stellar host star visual

Hero content:
- planet name
- host star name linked to `/stellarhosts/:hostname` when available
- short subtitle derived from available canonical values
- 3 to 4 hero stats
- generated planet visual

Preferred hero stats:
- records count
- discovery year
- radius
- mass

Fallback stats may use:
- orbital period
- equilibrium temperature
- discovery method

Hero subtitle should be compact and data-driven. Example shape:
- `Host star • radius • orbital period`

If values are missing, omit them rather than showing noisy placeholders.

### 2. Generated Planet Visual

Purpose:
- provide a deterministic visual anchor for the page
- mirror the role of `HostStarVisual` without pretending to be scientifically accurate

Rules:
- must be rendered locally in Leptos/CSS/SVG, not fetched from an external generator
- must be deterministic from planet data
- must remain stable between renders for the same summary values

Suggested visual inputs:
- equilibrium temperature or insolation: palette bias
- radius or mass: apparent scale within the hero composition
- density or bulk class: texture/ring/accent choices
- orbital period or semi-major axis: optional orbit accent styling

The output should communicate “planet archetype”, not measured appearance.

Implementation direction:
- create a `PlanetVisualTokens` struct similar in role to the star visual tokens
- derive a small visual class system such as rocky / sub-neptune / gas giant / hot world / cold world
- keep the visual logic isolated from page layout code

### 3. Canonical Summary Section

Purpose:
- show the adopted planet values computed from all available rows
- make disagreement visible without requiring the user to inspect every record

This section is the exoplanet analogue of stellar host canonical summary, but with planet fields.

Initial preferred fields:
- `pl_rade`
- `pl_bmasse` or `pl_masse` if only one is consistently available in the dataset used here
- `pl_dens`
- `pl_orbper`
- `pl_orbsmax`
- `pl_eqt`
- `discoverymethod`
- `disc_year`
- `hostname`

Field behavior:
- numeric fields use one adopted value plus evidence metadata
- categorical/stable fields show dominant or stable value plus disagreement signal when needed
- `hostname` should link to the matching stellar host detail route when available
- omitted fields should disappear cleanly if no useful data exists

For numeric summaries, show:
- adopted value
- unit
- measurement count
- distinct count
- min/max range when disputed

For stable/categorical summaries, show:
- selected value
- whether records agree
- distinct alternatives or counts where relevant

### 4. Scale Comparison Section

Purpose:
- graphically compare the current planet against familiar reference bodies

This section must be its own component and not a property-card variant.

Required references:
- Earth
- Jupiter

Required initial comparison basis:
- radius

Optional future comparison bases:
- mass
- density
- surface gravity

V1 behavior:
- use the adopted/canonical planet radius
- show side-by-side labeled bodies
- scale the exoplanet body relative to Earth and Jupiter with sensible min/max clamping so tiny or huge values remain readable
- if radius is unavailable, hide the section entirely

The component should make the comparison legible on mobile and desktop.

## Records And Provenance Section

Purpose:
- preserve direct access to all source rows
- expose row-level differences behind the canonical summary

V1 structure:
- section header with row count
- per-record cards
- expandable property table inside each card

Per-record card header should prefer:
- discovery method
- year
- facility or telescope/source label when available

The records section may later split into separate “provenance summary” and “raw records” subsections, but v1 can keep them combined if the information density stays readable.

## Data Contract

The current payload is too thin for the target page. The page should move to a richer backend contract instead of deriving everything from arbitrary first-row values.

### Current

```rust
pub struct ExoplanetDetail {
    pub pl_name: String,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}
```

### Target

```rust
pub struct ExoplanetDetail {
    pub pl_name: String,
    pub identity: ExoplanetIdentity,
    pub canonical: ExoplanetCanonicalSummary,
    pub visual: ExoplanetVisualSummary,
    pub provenance: ExoplanetProvenanceSummary,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}
```

Suggested supporting types:

```rust
pub struct ExoplanetIdentity {
    pub pl_name: String,
    pub hostname: Option<StableValueSummary>,
}

pub struct ExoplanetCanonicalSummary {
    pub radius: Option<NumericFieldSummary>,
    pub mass: Option<NumericFieldSummary>,
    pub density: Option<NumericFieldSummary>,
    pub orbital_period: Option<NumericFieldSummary>,
    pub semi_major_axis: Option<NumericFieldSummary>,
    pub equilibrium_temperature: Option<NumericFieldSummary>,
    pub discovery_method: Option<CategoricalFieldSummary>,
    pub discovery_year: Option<StableValueSummary>,
}

pub struct ExoplanetVisualSummary {
    pub radius_rearth: Option<f64>,
    pub mass_mearth: Option<f64>,
    pub density_cgs: Option<f64>,
    pub equilibrium_temperature_k: Option<f64>,
}

pub struct ExoplanetProvenanceSummary {
    pub record_count: usize,
    pub refs: Vec<String>,
    pub key_field_stats: Vec<ProvenanceStat>,
}
```

Notes:
- `visual` is a convenience layer for deterministic rendering inputs
- reuse existing summary types where possible instead of inventing parallel formatting models
- if `pl_bmasse` and `pl_masse` differ in semantics in the source data, the backend spec must choose one preferred canonical mass field and document the fallback rule

## Summary Computation Rules

The exoplanet canonical summary should follow the same broad philosophy already used for stellar hosts.

Numeric fields:
- ignore null values
- derive adopted value from the median of available measurements
- expose `measurement_count`, `distinct_count`, `min`, `max`, and `disputed`

Stable fields:
- use the shared stable summary model when all non-null values should normally agree
- mark as disputed when distinct values differ

Categorical fields:
- choose the most common value as the adopted display value
- keep counts for displayed disagreement context

The backend implementation spec should document exact field mapping and fallback behavior before code changes start.

## Styling

Add `style/components/exoplanet-detail.css` and import it from `style/tailwind.css`.

Rules:
- move repeated page and section utility stacks into semantic classes
- keep tiny one-off utilities inline only when that improves readability
- match the design family of `stellarhost_detail`
- preserve exoplanet-specific visual identity

Preferred naming pattern:
- `.exoplanet-detail-page__*`
- `.planet-hero__*`
- `.planet-visual__*`
- `.planet-summary__*`
- `.planet-comparison__*`
- `.planet-records__*`

## Refactor Plan

### Phase 1

- create this spec
- align on section model and backend target contract

### Phase 2

- split `src/components/exoplanet_detail.rs` into feature module files
- preserve current route and behavior as much as possible
- add semantic CSS and remove large inline class piles

### Phase 3

- upgrade backend payload from raw-row-only shape to canonical summary shape
- replace first-row summary logic with canonical summary rendering

### Phase 4

- add generated planet visual
- add Earth/Jupiter comparison section

## Open Questions

- Should the visual classification logic be fully deterministic from measured fields only, or can it include a small curated style mapping for better aesthetics?
- Which mass field is the canonical one for this app: `pl_bmasse`, `pl_masse`, or a documented preference order?
- Should discovery provenance references be surfaced as a dedicated summary panel before raw rows, or remain embedded in the records section for v1?
- Should the comparison section support toggling between radius and mass in v1, or stay radius-only until the canonical summary work is complete?
