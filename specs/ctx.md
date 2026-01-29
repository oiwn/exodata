# Current Context

## VOTable Refactor (planning only)

### Current breakages

- `crates/exo-core/src/tables.rs` has all modules commented out and the entire `crates/exo-core/src/tables/` folder was removed, breaking all `exo_core::tables::*` imports.
- Call sites currently broken: `src/main.rs`, `src/stellarhosts.rs`, `src/server/functions.rs`, `crates/exo-cli/src/main.rs`, `crates/exo-cli/src/commands.rs`, `examples/create_fixtures.rs`, `examples/*_inspection.rs`, `crates/exo-core/tests/*`.

Former "crates/exo-core/src/tables" files stored in "tmp/tables".

### Move into exo-cli (VOTable-only)

1. `crates/exo-core/src/tables/votable_loader.rs`
2. `crates/exo-core/src/tables/conversion.rs`
3. `crates/exo-core/src/common.rs` (VOTable helpers: headers, nullability, codegen)
4. VOTable parsing in `crates/exo-core/src/metadata.rs`:
   - `parse_votable_metadata`
   - `get_exoplanets_metadata`
   - `get_stellarhosts_metadata`
5. `examples/create_fixtures.rs` should be moved under CLI or updated to use the new CLI module.

### Keep or restore in exo-core

- `crates/exo-core/src/tables/common.rs` (parquet load + stats helpers)
- `crates/exo-core/src/tables/overview.rs` (to be feature-flagged as planned)
- `crates/exo-core/src/metadata.rs` should keep:
  - `ColumnMetadata`
  - TOML load/save: `save_metadata_toml`, `load_metadata_toml`
  - `get_columns_metadata` and `print_metadata` (optional to keep in core)

### Dependency shifts (planning)

- Remove from `exo-core`: `votable` (and possibly `indicatif` if no longer used).
- Add/keep in `exo-cli`: `votable` (keep `indicatif` for progress).

### Checklist

1. Restore `exo-core` table modules except VOTable-specific ones.
2. Move VOTable loader + conversion + VOTable helpers into `exo-cli`.
3. Split metadata: core keeps TOML + types; CLI owns VOTable parsing.
4. Update CLI calls:
   - `exo_core::tables::conversion` -> CLI module
   - `exo_core::common::print_votable_headers` -> CLI module
   - `exo_core::metadata::parse_votable_metadata` -> CLI module
5. Decide what to do with `examples/create_fixtures.rs`.
