# Project Overview

This project is a web application and CLI tool for browsing and analyzing exoplanet catalog data. The project is organized as a Cargo workspace with three main components:

## Architecture

1. **exoplanets-catalog** (Root) - Web application
   - Built with [Leptos](https://github.com/leptos-rs/leptos) (frontend) and [Axum](https://github.com/tokio-rs/axum) (backend)
   - Styled with [Tailwind CSS](https://tailwindcss.com/)
   - REST API at `/rest`, Swagger UI at `/swagger-ui`, SQL query endpoint at `/rest/query`
   - Provides interactive UI for browsing exoplanet data

2. **exo-core** (`crates/exo-core`) - Shared data processing library
   - Handles VOTable parsing and Parquet I/O
   - Provides DataFrame operations using [Polars](https://pola.rs/)
   - Statistical aggregations and analysis functions
   - Used by both the CLI and web app

3. **exo-cli** (`crates/exo-cli`) - Command-line tool
   - Standalone binary for data exploration and conversion
   - Built with [clap](https://github.com/clap-rs/clap)
   - Uses exo-core for all data operations

The project uses feature flags (`ssr` for server, `hydrate` for client) to separate server-side and client-side code in the web application.

# Building and Running

The following commands are the primary way to interact with the project.

## Web Application

**Development:**
```bash
cargo leptos watch    # Hot-reload development server
cargo leptos serve    # Development server without watch
```

**Production:**
```bash
cargo leptos build --release
# Output: target/server/release/exoplanets-catalog (server)
#         target/site/ (WASM + static assets)
```

## CLI Tool

**Development:**
```bash
cargo run --package exo-cli -- <command>
cargo run --package exo-cli -- --help
```

**Production:**
```bash
cargo build --package exo-cli --release
# Output: target/release/exo
```

**Examples:**
```bash
exo view-stats                              # View stellarhosts statistics
exo view-samples -l 20 -c stellar          # View stellar properties
exo view-exoplanets-samples -c orbital     # View orbital parameters
exo convert-raw-files                       # Convert VOTable to Parquet
```

## Testing

**Unit/Integration (Rust):**
```bash
cargo test                                   # All workspace tests
cargo test -p exo-core                       # exo-core only
cargo test -p exo-cli                        # exo-cli only
cargo test -p exoplanets-catalog --features ssr    # server/REST tests
```

**End-to-end (Playwright):**
The tests are located in the `end2end/tests` directory. To run the tests, use the following command:

```bash
cargo leptos end-to-end
```

## Data Management

The application includes a command to download exoplanet data.

```bash
just download-stellarhosts
```

This will download the data into the `data/` directory.

# Development Conventions

## Directory Structure

*   **`src/`** - Web application (Leptos + Axum)
    *   `main.rs` - Web server entry point
    *   `app.rs` - Leptos application and routing
    *   `components/` - UI components
    *   `server/` - Server functions and API handlers
    *   `tables/` - Data processing (local copy for web app)
*   **`crates/exo-core/`** - Shared data processing library
    *   Pure data processing logic (no CLI, no web)
    *   Used by both exo-cli and web app
*   **`crates/exo-cli/`** - CLI tool
    *   Command-line interface using exo-core
*   **`style/`** - Tailwind CSS configuration and styles
*   **`specs/`** - Technical specifications
    *   `overview.md` - Project status and links
    *   `cli.md` - CLI tool spec
    *   `data-management.md` - Data fetching and preparation
    *   `column-metadata.md` - Column descriptions and units
    *   `web-backend.md` - Axum server spec
    *   `web-frontend.md` - Leptos UI spec
    *   `ctx.md` - Current context / TODOs
    *   `ideas.md` - Short notes and ideas

## Code Organization

*   **Feature Flags:** The web app uses `ssr` (server) and `hydrate` (client) flags to separate code
*   **Styling:** Tailwind CSS with utility-first approach
*   **Testing:** Playwright E2E tests in `end2end/` directory
*   **Data:** Parquet files in `data/` directory

# Agent Rules

1.  **Explicit Instruction Compliance:** I will not perform any actions, including file modifications or command execution, unless I am explicitly asked to do so by the user. I will wait for a direct instruction before taking any action.
2.  **Confidence Threshold for Human-in-the-Loop:** If my confidence in understanding a request or predicting the outcome of an action is below a high threshold (e.g., 70%), I will immediately stop and ask the user for clarification or guidance. I will state what I am unsure about and why.

# Workflow

## Development Process

1. **Specification Development**: We develop specifications through an iterative process
   - Start with high-level requirements
   - Refine through implementation feedback
   - Update with insights gained from data analysis

2. **Specification Storage**: All specifications are stored in the `specs/` directory
   - Each major task has its own specification file
   - Specifications are versioned with git history
   - Use consistent markdown format

3. **Specification Requirements**: Specifications should be purely technical, not scientific
   - Focus on what code should do (implementation details)
   - Include just enough information to generate code
   - Avoid scientific analysis or domain-specific knowledge
   - Provide clear, actionable technical requirements

## Specification Guidelines

- **Technical Focus**: Specify functions, data structures, interfaces
- **Code Generation**: Include enough detail for implementation
- **Avoid Science**: Don't explain astrophysics, only what to implement
- **Iterative**: Update specs as implementation reveals insights
- **Examples**: Provide code examples and expected outputs
