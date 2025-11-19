# Exoplanets Catalog: Overview

This document outlines the structure, design, and architecture of the Exoplanets Catalog application.

## Structure

The project is organized into the following directories:

*   `src`: Contains the Rust source code for the application.
    *   `main.rs`: The main entry point for the application. It includes the CLI for data management and the code to start the web server.
    *   `app.rs`: Defines the main Leptos application component, including the UI and routing.
    *   `stellarhosts.rs`: Contains the logic for loading and processing the exoplanet data.
    *   `common.rs`: Shared utility functions.
    *   `fileserv.rs`: Handles serving static files.
*   `end2end`: Contains end-to-end tests written with Playwright.
*   `style`: Contains the CSS and SCSS files for styling the application.
*   `public`: Contains static assets like `favicon.ico`.
*   `data`: Intended to store data files, such as the `stellarhosts.vot` file.
*   `specs`: Contains project specification documents, like this one.

## Design

The application is designed with the following principles in mind:

*   **Frontend:** The user interface is built as a reactive single-page application (SPA) using the [Leptos](https://github.com/leptos-rs/leptos) framework. Styling is handled by [Tailwind CSS](https://tailwindcss.com/).
*   **Backend:** The web server is built with [Axum](https://github.com/tokio-rs/axum), a high-performance web framework for Rust.
*   **Data:** The exoplanet data is stored in the VOTable XML format.

## Architecture

The application follows a modern web architecture:

*   **Server-Side Rendering (SSR) with Hydration:** The application is rendered on the server for fast initial page loads and then "hydrated" on the client-side to enable full interactivity.
*   **Single Binary:** The entire application, including the web server and CLI, is compiled into a single binary. This simplifies deployment and execution.
*   **In-Memory Data:** The exoplanet data is loaded from the VOTable file into memory when the application starts. This allows for fast data access and filtering.
