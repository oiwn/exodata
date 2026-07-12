# Web Frontend Specification: Leptos UI

This document describes the Leptos-based frontend application for the exoplanets catalog.

## Overview

The frontend is a reactive single-page application (SPA) built with:
- **Leptos** - Reactive web framework
- **Tailwind CSS** - Utility-first styling
- **Server-Side Rendering (SSR)** - Initial page load from server
- **Hydration** - Client-side interactivity after SSR
- **WASM** - Client-side code compiled to WebAssembly

## Localization

The website uses `leptos_i18n` with compile-time JSON resources for English,
Simplified Chinese, and Japanese. English is the default and remains
unprefixed; `/zh-CN` and `/ja` select the other locales. Locale-prefixed aliases
exist for website routes, while REST, MCP, Swagger, sitemap, and export URLs
remain unprefixed.

The active locale is derived only from the URL. The language switcher preserves
the current path, query string, and fragment. In the first localization pass,
global navigation, the footer, and the homepage are translated; table, detail,
insight, and technical documentation content remains English.

## Architecture

```
src/
├── lib.rs                  # Library entry point (hydration)
├── app.rs                  # Main App component and routing
├── components/
│   ├── mod.rs              # Component module exports
│   └── overview.rs         # Overview page component
└── error_template.rs       # Error display component
```

## Component Structure

### 1. App Component (app.rs)

The root component that sets up routing and layout.

```rust
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/exoplanets-catalog.css"/>
        <Title text="Welcome to Leptos"/>

        <Router>
            <Routes fallback=|| { /* 404 handler */ }>
                <Route path=StaticSegment("") view=OverviewPage/>
                <Route path=StaticSegment("overview") view=OverviewPage/>
                <Route path=StaticSegment("table") view=TableWrapper/>
            </Routes>
        </Router>
    }
}
```

**Features:**
- Sets up meta context for `<Title>`, `<Meta>` tags
- Loads main stylesheet
- Defines client-side routes
- 404 fallback with error template

### 2. Shell Function (app.rs)

HTML document shell for SSR.

```rust
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
```

**Features:**
- HTML document structure
- Viewport configuration for mobile
- Auto-reload in development (hot module replacement)
- Hydration scripts for client interactivity
- Meta tags injection

### 3. OverviewPage Component (components/overview.rs)

Main dashboard displaying exoplanet statistics.

#### Component Structure

```rust
#[component]
pub fn OverviewPage() -> impl IntoView {
    // Create resource that fetches data from server
    let stats_resource = Resource::new(
        move || (),
        move |_| async move { get_stats().await },
    );

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            // Header with title
            <div class="relative overflow-hidden">
                <h1>"🌌 Exoplanet Archive"</h1>
                <p>"Exploring the cosmos..."</p>
            </div>

            // Main content with loading state
            <div class="container mx-auto px-4 pb-16">
                <Suspense fallback=LoadingSpinner>
                    {move || stats_resource.get().map(|result| {
                        // Display stats or error
                    })}
                </Suspense>
            </div>
        </div>
    }
}
```

#### Resource Pattern

Leptos `Resource` handles async data fetching:
- Tracks loading state automatically
- Triggers on dependency changes
- Integrates with `<Suspense>` for loading UI

```rust
let stats_resource = Resource::new(
    move || (),                          // Dependencies (none here)
    move |_| async move {                // Async fetcher
        get_stats().await                // Server function call
    },
);
```

#### Loading State

```rust
<Suspense fallback=move || {
    view! {
        <div class="flex flex-col justify-center items-center py-20">
            <div class="animate-spin rounded-full h-20 w-20 border-t-4 border-b-4 border-purple-500"></div>
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-4xl">
                "🪐"
            </div>
            <span class="mt-6 text-lg text-gray-300 animate-pulse">"Loading cosmic data..."</span>
        </div>
    }
}>
```

**Features:**
- Animated spinner with planet emoji
- Pulsing text
- Centered layout

#### Error Handling

```rust
stats_resource.get().map(|result| match result {
    Ok(stats) => leptos::either::Either::Left(view! {
        // Display stats
    }),
    Err(err) => leptos::either::Either::Right(view! {
        <div class="bg-red-900/50 border-2 border-red-500...">
            <span>"⚠️"</span>
            <h3>"Connection Error"</h3>
            <p>{format!("Error loading data: {}", err)}</p>
        </div>
    })
})
```

**Key Pattern:**
- Use `Either::Left` / `Either::Right` for different view types in match arms
- This solves type compatibility issues in match expressions

#### Stats Display

Two sub-components:

**StatsOverview** - High-level metrics:
```rust
#[component]
fn StatsOverview(stats: DataStats) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <StatCard
                icon="🌟"
                title="Stellar Hosts"
                value=stats.stellarhosts_total
                subtitle="Known systems"
            />
            <StatCard
                icon="🪐"
                title="Exoplanets"
                value=stats.exoplanets_total
                subtitle="Confirmed worlds"
            />
            <StatCard
                icon="🌡️"
                title="Avg Temperature"
                value=format!("{:.0} K", stats.avg_stellar_temp)
                subtitle="Stellar hosts"
            />
            <StatCard
                icon="📏"
                title="Avg Distance"
                value=format!("{:.1} pc", stats.avg_stellar_distance)
                subtitle="From Earth"
            />
        </div>
    }
}
```

**DetailedStats** - Discovery methods and size categories:
```rust
#[component]
fn DetailedStats(stats: DataStats) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            // Discovery methods chart
            <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-6">
                <h3>"🔭 Discovery Methods"</h3>
                <div class="space-y-3">
                    {stats.discovery_methods.iter().map(|(method, count)| {
                        // Bar chart visualization
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Planet size categories
            <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-6">
                <h3>"📊 Planet Size Distribution"</h3>
                // Similar bar chart for sizes
            </div>
        </div>
    }
}
```

#### StatCard Component

Reusable card for displaying metrics:
```rust
#[component]
fn StatCard(
    icon: &'static str,
    title: &'static str,
    value: String,
    subtitle: &'static str,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-6 border border-slate-700/50">
            <div class="flex items-center justify-between mb-4">
                <span class="text-4xl">{icon}</span>
            </div>
            <h3 class="text-sm font-medium text-gray-400 mb-1">{title}</h3>
            <p class="text-3xl font-bold text-white mb-1">{value}</p>
            <p class="text-xs text-gray-500">{subtitle}</p>
        </div>
    }
}
```

## Routing

Routes configured in `App` component:

```rust
<Route path=StaticSegment("") view=OverviewPage/>         // /
<Route path=StaticSegment("overview") view=OverviewPage/>  // /overview
<Route path=StaticSegment("table") view=TableWrapper/>     // /table
```

**Navigation:**
```rust
// Programmatic navigation
let navigate = use_navigate();
navigate("/overview", Default::default());

// Link component
<A href="/overview">"Go to Overview"</A>
```

## Server Function Integration

### Calling Server Functions

```rust
use crate::server::functions::get_stats;

// In component
let stats_resource = Resource::new(
    move || (),
    move |_| async move { get_stats().await },
);
```

### Data Structures

Must be defined on both client and server:

```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    pub stellarhosts_total: usize,
    pub exoplanets_total: usize,
    pub avg_stellar_temp: f64,
    pub avg_stellar_distance: f64,
    pub discovery_methods: Vec<(String, usize)>,
    pub planet_size_categories: Vec<(String, usize)>,
}
```

**Feature Flags:**
```rust
// Server-side definition
#[cfg(feature = "ssr")]
use crate::server::functions::DataStats;

// Client-side definition
#[cfg(not(feature = "ssr"))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats { /* same fields */ }
```

## Styling with Tailwind CSS

### Configuration

Tailwind CSS v4 is configured through `style/tailwind.css`; no JavaScript
Tailwind config file is required.

### Input File (style/tailwind.css)

```css
@import "tailwindcss";
```

### Usage in Components

```rust
view! {
    <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
        <h1 class="text-5xl md:text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 animate-pulse">
            "🌌 Exoplanet Archive"
        </h1>
    </div>
}
```

### Common Patterns

**Responsive Grid:**
```rust
class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6"
```

**Glassmorphism Cards:**
```rust
class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-6 border border-slate-700/50"
```

**Gradient Text:**
```rust
class="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400"
```

**Loading Spinner:**
```rust
class="animate-spin rounded-full h-20 w-20 border-t-4 border-b-4 border-purple-500"
```

## Reactivity

### Signals

Simple reactive values:
```rust
let (count, set_count) = signal(0);

// Read
let value = count.get();

// Write
set_count.set(10);

// Update
set_count.update(|n| *n += 1);
```

### Resources

Async data fetching:
```rust
let data = Resource::new(
    move || dependency_signal.get(),  // Refetches when this changes
    move |dep_value| async move {
        fetch_data(dep_value).await
    },
);
```

### Memos

Derived reactive values:
```rust
let doubled = Memo::new(move |_| count.get() * 2);
```

### Effects

Side effects that run when dependencies change:
```rust
Effect::new(move |_| {
    println!("Count changed to: {}", count.get());
});
```

## Hydration

### How It Works

1. **Server renders** initial HTML with data
2. **Browser loads** HTML (fast first paint)
3. **WASM downloads** and executes
4. **Leptos hydrates** - attaches event listeners to existing DOM
5. **Fully interactive** - client-side routing and reactivity enabled

### Hydration Scripts

Automatically injected by `HydrationScripts`:
```rust
<HydrationScripts options=options.clone() />
```

This loads:
- WASM module (`/pkg/exoplanets-catalog_bg.wasm`)
- JS glue code (`/pkg/exoplanets-catalog.js`)
- Hydration data (serialized state)

## Build Process

### Development

```bash
cargo leptos watch
```

Outputs:
- Server binary (SSR)
- WASM module (client)
- Watches for file changes
- Hot module replacement

### Production

```bash
cargo leptos build --release
```

Outputs:
- `target/server/release/exoplanets-catalog` (server binary)
- `target/site/pkg/` (optimized WASM and JS)
- `target/site/` (static assets)

### Feature Flags

- **`ssr`**: Server-side code (Axum, server functions)
- **`hydrate`**: Client-side code (WASM, event handlers)

Compilation:
```bash
# Server binary (ssr feature)
cargo build --features ssr

# WASM client (hydrate feature)
cargo build --target wasm32-unknown-unknown --features hydrate
```

## Performance Optimization

### WASM Size

Configured in `Cargo.toml`:
```toml
[profile.wasm-release]
inherits = "release"
opt-level = 'z'        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
panic = "abort"        # Smaller binary
```

### Code Splitting

Future: Split WASM into chunks for faster initial load.

### Lazy Loading

Future: Load components on-demand.

## Accessibility

### Semantic HTML

Use proper semantic elements:
```rust
<main>
  <header>
    <h1>...</h1>
  </header>
  <section>...</section>
</main>
```

### ARIA Labels

Add labels for screen readers:
```rust
<button aria-label="Close dialog">
  "×"
</button>
```

### Keyboard Navigation

Ensure interactive elements are keyboard accessible.

## Testing

### Unit Tests

Test components in isolation:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_card() {
        // Test component rendering
    }
}
```

### E2E Tests

Playwright tests in `end2end/` directory.

## Future Enhancements

### Components

- Table view with pagination
- Search and filter UI
- Detail pages for individual stars/planets
- Interactive charts (plotly, d3)
- Dark/light theme toggle

### Features

- Client-side caching
- Offline support (service worker)
- Progressive Web App (PWA)
- Export data (CSV, JSON)
- Bookmarking/favorites
- Comparison tool

### Performance

- Virtual scrolling for large tables
- Image lazy loading
- Route-based code splitting
- Prefetching data

### Accessibility

- Keyboard shortcuts
- Screen reader optimization
- High contrast mode
- Reduced motion support
