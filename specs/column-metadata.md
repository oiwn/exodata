# Column Metadata

Metadata describes catalog columns independently of paginated table responses.

## Ownership

| Layer | Responsibility |
| --- | --- |
| [exo-types metadata](../crates/exo-types/src/metadata.rs) | Shared serializable `ColumnMetadata` |
| [CLI VOTable helpers](../crates/exo-cli/src/votable_helpers.rs) | Parse VOTable FIELD metadata from a file path |
| [CLI conversion](../crates/exo-cli/src/conversion.rs) | Generate Parquet and matching metadata TOML |
| [exo-core metadata](../crates/exo-core/src/metadata.rs) | TOML save/load and description formatting helpers |
| [UI metadata store](../src/metadata.rs) | Shared metadata for SSR and hydration |
| [REST handlers](../src/server/handlers.rs) | Schema responses combining data types and source metadata |

`ColumnMetadata` contains `name: String`, `description: Option<String>`,
`unit: Option<String>`, and `datatype: String`. Source names and units remain
unchanged; interface localization does not translate NASA keys or values.

## Extraction and Persistence

`parse_votable_metadata(vot_path: &str)` reads FIELD definitions through the
VOTable iterator and returns a map keyed by field name. Descriptions and units
are optional; datatype is the parser's debug-formatted source datatype.

The converter writes `<stem>-metadata.toml` alongside each Parquet file.
TOML uses a `[[column]]` array with the fields above, sorted by name for stable
serialization. Loading reconstructs the name-keyed map. File sizes and column
counts depend on the source export and are not fixed requirements.

See [data-management.md](data-management.md) for artifact contracts and the
[data skill](../.agents/skills/exodata-data/SKILL.md) for inspection/conversion.

## Runtime Flow

1. Startup loads both metadata TOML files into `ApiState`.
2. It creates `AppMetadata { stellarhosts, exoplanets }`, provides it to SSR
   context, and embeds JSON in the shell's `__EXO_METADATA__` script.
3. `provide_app_metadata_store` reuses an existing store, otherwise initializes
   it from SSR context or the embedded hydration payload. If neither is
   available, it provides an empty default.
4. Table pages read the shared store for column selection and rendering.
   `TableData` contains rows, columns, totals, page, and limit, without metadata.

[Table headers](../src/table/table.rs) already display description/unit
tooltips. [Column selection](../src/components/column_selector.rs) exposes
metadata descriptions and units and excludes error/limit companion columns
from its selectable list. Grouping/display behavior lives in
[column_model.rs](../src/table/column_model.rs).

Detail payloads still include metadata explicitly. REST schema endpoints expose
actual Polars column types plus available source descriptions, units, and
datatypes; they do not rely on the UI store.

## Verification

Existing metadata tests cover description/unit formatting and TOML round trips.
VOTable metadata extraction currently has no dedicated parser test. Browser
smoke coverage includes metadata availability after homepage-to-table client navigation.
See [testing.md](testing.md); exact fixture sizes are not production contracts.

Potential additional FIELD attributes and metadata completeness checks remain
ideas, not current payload requirements.
