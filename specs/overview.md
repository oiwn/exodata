# Exoplanets Catalog: Project Documentation

This file has been replaced by more detailed specifications. Please refer to:

## Architecture & Project Structure
See **`architecture.md`** for workspace organization, build commands, and overall system design.

## Component Specifications

1. **`data-layer.md`** - exo-core library (data processing, VOTable, Parquet, aggregations)
2. **`cli.md`** - exo-cli command-line tool (commands, usage, examples)
3. **`web-backend.md`** - Axum server (REST API, server functions, state management)
4. **`web-frontend.md`** - Leptos UI (components, routing, styling, reactivity)
5. **`ideas.md`** - short notes with ideas
6. **`data-management.md`** - how to fetch and prepare the data
7. **`column-metadata.md`** - information about each column

## Quick Start

**Web Application:**
```bash
cargo leptos watch    # Development
cargo leptos build --release    # Production
```

**CLI Tool:**
```bash
cargo run --package exo-cli -- --help
cargo run --package exo-cli -- view-stats
```

For complete documentation, see the specification files listed above.
