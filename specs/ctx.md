# Current Context

## Next Refactor

Planned frontend split for stellar host detail:

- create `src/components/stellarhost_detail/` as a dedicated submodule
- move the current route/container logic into `page.rs`
- move hero layout into `hero.rs`
- move the star art into `star_visual.rs`
- move canonical summary cards into `summary.rs`
- move planets section into `planets.rs`
- move provenance section into `provenance.rs`
- move shared labels/formatting helpers into `format.rs`

Target ownership:

- `page.rs`
  - route-level resource wiring only
- `hero.rs`
  - host identity copy, hero stats, alias chips, overall hero layout
- `star_visual.rs`
  - decorative/data-driven star rendering only
- `summary.rs`
  - canonical summary section and summary cards
- `planets.rs`
  - known-planets section and planet cards
- `provenance.rs`
  - evidence summary, observations table, reference-link rendering
- `format.rs`
  - shared formatting helpers and label helpers

Reason for the split:

- the current detail-page file has become CSS-heavy and section-dense
- hero, provenance, and summary concerns are now large enough to evolve independently
- the star visual should be isolated from text/layout concerns
- future visual iterations will be safer if each section owns its own markup and helper logic

## Stellar Hosts

### Product Rule

- `stellarhosts` table view remains a raw NASA-style record browser.
- Stellar host detail view is a canonical per-`hostname` profile derived from all matching rows.
- Raw source rows remain visible on the detail page as provenance, but they are secondary to the canonical summary.

### Detail Page Shape

The stellar host detail page should have three layers:

1. Identity / hero
   - `hostname`
   - adopted spectral type if available
   - adopted distance if available
   - planet count if available
   - strong visual star hero
2. Canonical summary
   - adopted values for key stellar/system properties
   - disagreement shown via range, distinct count, and measurement count
3. Provenance
   - compact evidence summary
   - compact observations table
   - source/reference links

### Canonicalization Rules

#### Identity

- `hostname` is the grouping key and canonical identifier.
- `hd_name`, `hip_name`, and `tic_id` are aliases.
- Aliases are collected as distinct non-null values.

#### Stable System Fields

- Applies to `sy_pnum`, `sy_snum`, `sy_mnum`.
- If one non-null distinct value exists, use it directly.
- If multiple distinct values exist, mark the field as disputed and expose all distinct values in provenance.

#### Numeric Summary Fields

- Applies to `sy_dist`, `sy_plx`, `st_teff`, `st_mass`, `st_rad`, `st_age`, `st_lum`, `st_logg`, `st_met`.
- Canonical value is the median of non-null values.
- Provenance should retain:
  - measurement count
  - distinct count
  - min
  - max
  - disputed state
- If all values are null, omit the field from the main summary.

#### Categorical Summary Fields

- Applies to `st_spectype`.
- Canonical value is the most common non-null value.
- Provenance should retain distinct values and counts.
- If multiple values exist, mark the field as disputed.

### Provenance Rules

- Keep one row per raw source record for the selected `hostname`.
- First-version observations table columns:
  - `st_teff`
  - `st_mass`
  - `st_rad`
  - `st_age`
  - `st_lum`
  - `st_spectype`
  - `sy_dist`
  - `st_refname`
  - `sy_refname`
- Reference fields should render as links when the dataset provides archive anchor markup.

### In-Memory Rule

- Do not build a second global canonical stellar-host dataset.
- Keep the raw dataframe in memory.
- Derive canonical host detail on demand per `hostname`.
- Cache derived detail payloads per host.

### Visual Rule

- Detail page should feel like a host profile, not a raw record dump.
- The hero may use an approximate illustrative star color derived from `st_teff`.
- Wording should frame this as approximate, not exact visible appearance.

### Component Decomposition

- Keep the route container thin.
- Keep section-level layout in `stellarhost_detail_sections`.
- Move the hero star visual into its own component when the next hero pass happens.

Planned split:

- `HostHeroSection`
  - owns hero copy, adopted identity values, alias chips, and hero layout
- `HostStarVisual`
  - owns the decorative/illustrative star rendering only
  - takes already-derived display inputs such as approximate color, size/emphasis, and optional label text
  - should not know about raw records or canonicalization rules

Rationale:

- the star visual is presentation-only and should not be mixed with hero text/layout concerns
- isolating it makes future visual iterations safer
- this also makes it easier to swap the current static art for a more data-driven rendering later

Current action:

- keep the current implementation working
- plan the next refactor around extracting the star visual into a dedicated component file or subcomponent

### Status

- Canonical stellar host detail implementation exists in code.
- `specs/web-frontend.md` and `specs/web-backend.md` do not yet need duplication of this detail-page behavior unless the design stabilizes further.
