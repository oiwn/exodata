# Current Task Context: Add overview mass and stellar-class blocks

State: in progress (implementation complete; awaiting manual UI verification)

## Plan

- [x] Add and test distinct-planet mass and distinct-host stellar-class aggregations.
- [x] Expose and render both datasets in the second detailed-statistics row with translations.
- [x] Update the homepage specification and verify the affected code.

## Context

Issue #120 adds planet best-mass bands and leading spectral classes immediately
after Planet Classifications and Orbital Periods. The stellar-class block is
limited to the five most common classes so it ends at class A in the current
dataset.

## Next

Run `cargo leptos watch`, open `/`, and confirm the new second row at desktop
and narrow viewport widths.
