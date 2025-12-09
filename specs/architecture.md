# Exoplanets Catalog: Architecture

This document describes the overall workspace architecture and how the different crates work together.

## Workspace Structure

The project is organized as a Cargo workspace with three main components:

```
exoplanets-catalog/
├── Cargo.toml                    # Root workspace configuration
├── src/                          # Web application (Axum + Leptos)
│   ├── main.rs                   # Web server entry point
│   ├── lib.rs                    # Leptos library
│   ├── app.rs                    # Application routing
│   ├── components/               # UI components
│   └── server/                   # Server functions & handlers
├── crates/
│   ├── exo-core/                 # Shared data processing library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── common.rs         # VOTable utilities
│   │       └── tables/           # Data processing modules
│   │
│   └── exo-cli/                  # CLI tool binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands.rs
├── data/                         # Data files (parquet, VOTable)
├── style/                        # Tailwind CSS
├── public/                       # Static assets
└── specs/                        # Technical specifications

```

## Architecture Principles

### 1. Separation of Concerns
- **exo-core**: Pure data processing logic (no CLI, no web)
- **exo-cli**: Command-line interface using exo-core
- **exoplanets-catalog**: Web application using exo-core

### 2. Shared Core Library
The `exo-core` library contains all data processing logic:
- VOTable parsing and conversion
- Parquet file I/O
- DataFrame operations with Polars
- Data aggregations and statistics
- Shared utilities

Both the CLI and web app depend on `exo-core` for data operations.

### 3. Independent Deployment
- **CLI**: Build standalone with `cargo build --package exo-cli`
- **Web**: Build with `cargo leptos build` for production deployment
- **Core**: Library only, not deployable

## Crate Dependencies

```
exoplanets-catalog (web)
    └─> exo-core

exo-cli
    └─> exo-core

exo-core
    ├─> polars (data processing)
    ├─> votable (XML parsing)
    ├─> serde (serialization)
    └─> indicatif (progress bars)
```

## Build Commands

### Development

**Web application:**
```bash
cargo leptos watch        # Development with hot reload
cargo leptos serve        # Development server
```

**CLI:**
```bash
cargo run --package exo-cli -- --help
cargo run --package exo-cli -- view-stats
```

**Core library:**
```bash
cargo check --package exo-core
```

### Production

**Web application:**
```bash
cargo leptos build --release
# Output: target/server/release/exoplanets-catalog (server binary)
#         target/site/ (static assets, WASM)
```

**CLI:**
```bash
cargo build --package exo-cli --release
# Output: target/release/exo
```

## Feature Flags

The web application uses feature flags to separate server and client code:

- `ssr`: Server-side rendering features (Axum, server functions)
- `hydrate`: Client-side hydration (WASM)
- `default = ["ssr"]`: Server features enabled by default

## Data Flow

### CLI Workflow
1. User runs CLI command
2. exo-cli parses command with clap
3. Calls exo-core functions for data processing
4. Displays results in terminal (tables, stats)

### Web Workflow
1. Browser requests page → Axum server
2. Server loads data into memory at startup (ApiState)
3. Leptos renders initial HTML (SSR)
4. Client hydrates and becomes interactive
5. User interactions call server functions
6. Server functions access ApiState (no HTTP self-requests)
7. Results returned to client

## Key Design Decisions

### Why Workspace?
- **Smaller deployments**: Web binary doesn't include CLI dependencies
- **Clear boundaries**: Each crate has focused responsibility
- **Reusability**: Core library used by both CLI and web
- **Independent testing**: Test each component separately

### Why Keep Web App in Root?
- Leptos tooling (cargo-leptos) expects specific structure
- Easier configuration with existing Leptos.toml
- Root-level assets (style/, public/) work out of the box

### In-Memory Data Loading
- Data loaded once at server startup
- Shared via Arc<DataFrame> in ApiState
- Fast access, no disk I/O per request
- Trade-off: Higher memory usage, but acceptable for dataset size

## Directory Layout

| Directory | Purpose |
|-----------|---------|
| `src/` | Web application source (Leptos + Axum) |
| `crates/exo-core/src/` | Data processing library |
| `crates/exo-cli/src/` | CLI application |
| `data/` | Parquet and VOTable data files |
| `style/` | Tailwind CSS configuration and styles |
| `public/` | Static assets (favicon, images) |
| `specs/` | Technical specifications (this document) |
| `target/` | Build artifacts |
| `target/site/` | Leptos build output (server + WASM) |

## Next Steps

For detailed information about each component:
- **Data Processing**: See `specs/data-layer.md`
- **CLI Tool**: See `specs/cli.md`
- **Web Backend**: See `specs/web-backend.md`
- **Web Frontend**: See `specs/web-frontend.md`
