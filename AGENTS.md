# Project Overview

This project is a web application for browsing a catalog of exoplanets. It is built with the [Leptos](https://github.com/leptos-rs/leptos) web framework and uses [Axum](https://github.com/tokio-rs/axum) as the web server. The frontend is styled with [Tailwind CSS](https://tailwindcss.com/).

The application has two main parts:
1.  A web server that provides the main user interface for browsing the exoplanet data.
2.  A command-line interface (CLI) for managing the application's data, including importing data from external sources.

The project is written in Rust and uses a feature flag system to distinguish between server-side rendering (`ssr`) and client-side hydration (`hydrate`) code.

# Building and Running

The following commands are the primary way to interact with the project.

## Development

To run the application in development mode with hot-reloading, use the following command:

```bash
cargo leptos watch
```

## Production Build

To build the application for production, use the following command:

```bash
cargo leptos build --release
```

This will create a server binary in `target/server/release` and the static site assets in `target/site`.

## Testing

The project uses [Playwright](https://playwright.dev/) for end-to-end testing. The tests are located in the `end2end/tests` directory. To run the tests, use the following command:

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

*   **Main Application Logic:** The primary application logic for the Leptos frontend is located in `src/app.rs`.
*   **Server and CLI:** The main entry point for the application is `src/main.rs`, which handles both starting the web server and running the CLI commands.
*   **Styling:** The project uses Tailwind CSS for styling. The main CSS file is `style/tailwind.css`, and the configuration is in `tailwind.config.js`.
*   **End-to-End Tests:** End-to-end tests are written using Playwright and are located in the `end2end` directory.
*   **Feature Flags:** The project uses the `ssr` and `hydrate` feature flags to separate server-side and client-side code.

# Agent Rules

1.  **Explicit Instruction Compliance:** I will not perform any actions, including file modifications or command execution, unless I am explicitly asked to do so by the user. I will wait for a direct instruction before taking any action.
2.  **Confidence Threshold for Human-in-the-Loop:** If my confidence in understanding a request or predicting the outcome of an action is below a high threshold (e.g., 70%), I will immediately stop and ask the user for clarification or guidance. I will state what I am unsure about and why.