# Current Context

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
- The hero should use an approximate illustrative star color derived from canonical `st_teff` when available.
- Wording should frame this as approximate, not exact visible appearance.

### Star Color

#### Product Framing

- Treat star color as illustrative display logic, not exact astrophysical appearance.
- Use it to improve hero identity and visual differentiation between hosts.
- Product wording should remain:
  - `Approximate color from effective temperature`

#### Data Input

- Use canonical host temperature, not raw row values.
- Primary input is canonical `st_teff`.
- If `st_teff` is missing, fall back to a neutral default hero color.
- Do not use source-recency logic for color selection.

#### V1 Mapping Strategy

- Prefer a hand-tuned temperature-to-color scale over pure blackbody rendering.
- Use anchor temperatures with interpolation between anchors.
- Keep solar-like stars near pale yellow-white rather than heavily saturated yellow.
- Keep very hot stars blue-white, not neon blue.
- Keep cool stars orange-red, but avoid oversaturated red UI.

Suggested display bands:

- `< 3500 K`: deep orange-red
- `3500-4500 K`: orange
- `4500-5300 K`: warm yellow-orange
- `5300-6000 K`: pale yellow-white
- `6000-7500 K`: warm white
- `7500-10000 K`: blue-white
- `> 10000 K`: pale icy blue

#### Rendering Rule

- Do not use one raw color value everywhere.
- Derive multiple presentation tokens from the mapped color:
  - core star color
  - outer glow color
  - subtle panel/accent tint
  - near-white highlight

This keeps the hero visually controlled and avoids harsh or cartoonish output.

#### Non-Goals For V1

- do not attempt exact visible-color simulation
- do not add `st_logg` or metallicity into color rendering yet
- do not block the feature on scientific-model integration

#### Future Option

- If needed later, replace the curated mapping with a more physically informed model.
- That should only happen if the visual result is clearly better and still product-usable.

### Status

- Canonical stellar host detail and component split exist in code.
- Star-color rendering from canonical `st_teff` exists in code.
- `specs/web-frontend.md` and `specs/web-backend.md` do not yet need duplication of this detail-page behavior unless the design stabilizes further.
