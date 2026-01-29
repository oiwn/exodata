# Code Audit and Refactor Notes
**Date:** 2026-01-29  
**Auditor:** Rebuilt review (per user guidance)

## Executive Summary
The codebase has a mostly clear split between web/API shaping and shared data processing, but there are a few duplication points and future features that should be explicitly gated. The most immediate value is in clarifying intent (feature flags) and reducing duplicated logic/types.

## 1) Experimental Overview Analytics
**Location:** `crates/exo-core/src/tables/overview.rs`

Functions currently unused:
- `temperature_distribution`
- `discovery_timeline`
- `catalog_crossmatch`
- `photometric_statistics`

!!! These are intended to be used, so they are not "hallucinated." They should be placed behind an explicit feature flag to make intent clear and avoid compile time bloat until wired.

**Recommendation:** Gate these functions under a feature like `overview-advanced` (name TBD) and only enable in the web app when needed.

^^^ i removed them into different location outsize current repo.

## 2) VOTable Utilities Ownership
**Locations:** `src/common.rs`, `crates/exo-core/src/common.rs`

!!! VOTable helpers should live solely in the CLI module, since they are only needed for conversion into Parquet and metadata.

**Recommendation:** Move VOTable helpers into `crates/exo-cli` (or a CLI-only module) and remove the web app copy. Decide whether `exo-core/src/common.rs` is still necessary once the CLI owns these helpers.

## 3) ColumnMetadata Duplication
**Location:** `src/server/functions.rs`

There is a local `ColumnMetadata` type that duplicates `exo_core::metadata::ColumnMetadata` to keep the WASM bundle small.

!!! This is a deliberate client-bundle constraint, not a circular dependency issue. The audit should reflect that.

**Recommendation:** Extract a small "shared types" crate or module with serde-only types that can be used by both server and client without pulling in heavy dependencies.

## 4) Server Data Serialization Safety
**Location:** `src/server/common.rs` -> `dataframe_to_json`

The implementation matches on column dtype and returns `null` for unsupported types. There are no runtime `unwrap()` calls in production code.

!!! The previous audit's "panic-prone" claim is not supported. The risk is silent nulls for unsupported types, not crashes.

**Recommendation:** Add explicit handling for common missing types (Boolean, Date/Datetime, Categorical, List) or return a structured error when encountering unsupported types.

## 5) Duplicate Table Data Logic
**Location:** `src/server/common.rs`

`get_stellarhosts_data` and `get_exoplanets_data` share most of their logic (pagination, selection, sorting, JSON conversion).

!!! This is a refactoring target to prevent drift and reduce maintenance.

**Recommendation:** Extract a shared helper for the common pipeline and keep dataset-specific defaults separate.

## 6) Root-Level Housekeeping (Low Priority)
**Files:** `.tmuxp.yaml`, `notes.org`, `README_LEPTOS.md`

These are non-essential files in repo root. Low priority cleanup.

## Refactoring Targets (Planning)
1. Feature-flag advanced overview analytics in `crates/exo-core/src/tables/overview.rs`.
2. Move VOTable helpers into `crates/exo-cli`; remove the web app copy; re-evaluate `exo-core/src/common.rs`.
3. Create a shared serde-only types module/crate for `ColumnMetadata`.
4. Extract a generic table-data pipeline helper for pagination/sorting/selection/JSON conversion.
5. Expand `dataframe_to_json` to handle more dtypes or return structured errors.
