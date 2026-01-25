# Exoplanets Catalog: Project Documentation

## Current Status

**Latest Update**: REST API SQL query endpoint + Swagger UI ✅
- SQL endpoint: `GET /rest/query?sql=SELECT...`
- OpenAPI JSON: `GET /rest/openapi.json`
- Swagger UI: `GET /swagger-ui`

**Deployment Status**: Production deployment completed ✅
- Live at https://exodata.space
- GitHub Actions builds Docker image on version bump
- Ansible deploys to DigitalOcean droplet
- See `DEPLOY.md` in project root for deployment guide

---

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

## Deployment

See **`DEPLOY.md`** in project root for deployment guide (GitHub Actions, Ansible, DigitalOcean).

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

**Deployment:**
```bash
just ansible-deploy      # Deploy after image build
just ansible-status      # Check server status
just ansible-logs        # View container logs
```

For complete documentation, see the specification files listed above.
