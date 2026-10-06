---
id: 0003-homepage-mcp-description
status: approval
scope: [style/components/homepage-manual.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/148
attempts: 3
---
# Task: homepage mcp description

## Plan

- [x] Replace the homepage MCP manual, SQL card, example grid, and setup commands with a localized heading, brief factual description, and one locale-aware setup-guide link; preserve existing homepage fragment destinations.
- [x] Update English, Chinese, and Japanese homepage titles/descriptions using existing locale resources; keep the catalog hero, statistics, and browsing routes.
- [x] Limit CSS changes to the simplified section and document the homepage content contract; verify formatting, locale JSON, and the split SSR/hydration build before handing over manual review.

## Acceptance

- [x] `cargo fmt --all` and `git diff --check` pass for the scoped changes.
- [x] All three locale JSON files parse and supply the summary heading, description, and setup-link label; the component uses `localized_path` and retains `mcp-exoplanet-data`, `catalog-examples-title`, and `mcp-setup-title`.
- [x] `cargo leptos build --split` passes for SSR and hydration.
- [x] `specdev check` reports no errors or new task-specific warnings. Prior approved task changes in this uncommitted batch remain owned by their existing task scopes.

## Manual checks

- [ ] Run `EXO_DATA_DIR=end2end/runtime-data cargo leptos watch --split`; open `http://127.0.0.1:3000/` and confirm the compact MCP description/link, retained catalog statistics/navigation, and updated title/description.
- [ ] Follow the setup link to `/docs/mcp`, inspect the guide and canonical metadata, and check `/sitemap-static.xml` for the English guide entry.
- [ ] Check `/zh-CN` and `/ja`, language switching, existing homepage fragments, and the setup link's locale prefix; inspect the brief section on mobile for overflow.

## Summary

Replace the homepage MCP manual with a brief localized introduction and setup link, and improve homepage discovery metadata.

## Context

- Grouped scope globs support the uncommitted batch: new homepage edits are limited to `src/components/homepage_manual.rs`, `style/components/homepage-manual.css`, and the three existing locale JSON files. `docs/mcp.md`, `src/components/docs/registry.rs`, `src/server/handlers.rs`, and `src/server/tests.rs` are retained task 0002 changes, not additional homepage edits. Other files matched by the globs remain outside this plan. Task 0002 is parked as blocked pending manual review/merge under the existing batch convention.
- The user approved implementing the 2026-10-06 SEO/GEO plan and stopping at local manual review. Task 0002 owns the guide and sitemap; this task owns the homepage changes. Deployment, external accounts, crawler investigation, and analytics remain deferred.
- Use short translated strings in `locales/`; the existing long homepage Markdown files are retained as source material but are no longer rendered by the simplified component. Remove only Rust helpers and CSS made unused by this change.

## Findings

- Headline correction approved on 2026-10-07: issue #148 explicitly requests MCP in the homepage headline. Changed the English heading to “Exoplanet MCP server for AI agents” and updated Chinese/Japanese equivalents. This resolves the missed wording requirement; description and setup link remain as reviewed.
- Headline verification: all three locale JSON files parse and their headings include MCP; split SSR/hydration build and `git diff --check` passed on 2026-10-07. Refresh the homepage to check the updated heading and wrapping.
- OpenCode setup follow-up on 2026-10-07: installed `opencode mcp add --help` confirms the named server, `--url`, and optional `--global` flags. Added the user's exact command to the guide's quick setup and retained JSON as an alternative. Earlier verification covered the provider's JSON example but missed this CLI shortcut. Only help was executed; no MCP configuration was written. This explicitly requested documentation correction extends the guide file already included in the task's batch scope.
- Manual feedback on 2026-10-07: the user approved the block's appearance but requested less top/bottom padding. Reduced section vertical padding from `py-8` (2rem) to `py-4` (1rem); horizontal padding and content spacing are unchanged. Refresh the homepage to review the tighter block.
- Padding follow-up verification on 2026-10-07: split SSR/hydration and Tailwind build passed; `git diff --check` passed. The tighter block awaits visual confirmation.
- `cargo leptos build --split` passed on 2026-10-06 for SSR, hydration, WASM splitting, and Tailwind. `cargo fmt --all`, locale JSON/required-string checks, fragment/link structure checks, and `git diff --check` passed. `specdev check` reports zero errors and 40 existing warnings, with none on tasks 0002/0003 or ctx.
- The related task 0002 sitemap regression failed before `/docs/mcp` was added and passed afterward. No local server, browser automation, deployment, or client configuration write was performed; rendered metadata, language switching, guide navigation, and mobile layout await the user's manual checks above.

## Review

- Reviewed against the approved plan: homepage MCP content is one heading, a short factual paragraph, and one setup link; the component no longer renders a second H1, client commands, SQL card, or example grid. CSS removals correspond to those removed components. Homepage statistics and hero remain in their existing component.
- Locale resources include the brief copy and discovery metadata in all three languages; setup links use the existing path helper, and legacy fragments resolve into the compact section. Guide/sitemap work remains owned by task 0002; grouped scope exists only to validate the authorized uncommitted batch. Ready for manual review after the successful split build.

## Log

- 2026-10-05 created: homepage mcp description
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/148
- 2026-10-06 scope add src/components/homepage_manual.rs — Approved brief localized MCP introduction and homepage metadata
- 2026-10-06 scope add style/components/homepage-manual.css — Approved brief localized MCP introduction and homepage metadata
- 2026-10-06 scope add locales/en.json — Approved brief localized MCP introduction and homepage metadata
- 2026-10-06 scope add locales/zh-CN.json — Approved brief localized MCP introduction and homepage metadata
- 2026-10-06 scope add locales/ja.json — Approved brief localized MCP introduction and homepage metadata
- 2026-10-06 advance draft → ready
- 2026-10-06 scope approved: src/components/homepage_manual.rs, style/components/homepage-manual.css, locales/en.json, locales/zh-CN.json, locales/ja.json
- 2026-10-06 advance ready → in-progress/implement
- 2026-10-06 blocked: Adjusting scope for the authorized uncommitted task 0002/0003 batch before homepage edits
- 2026-10-06 scope rm src/components/homepage_manual.rs
- 2026-10-06 scope rm locales/en.json
- 2026-10-06 scope rm locales/zh-CN.json
- 2026-10-06 scope rm locales/ja.json
- 2026-10-06 scope add src/components/**/*.rs — Grouped approved homepage files plus retained task 0002 changes in the uncommitted batch; exact intended edits recorded in Context
- 2026-10-06 scope add src/server/*.rs — Grouped approved homepage files plus retained task 0002 changes in the uncommitted batch; exact intended edits recorded in Context
- 2026-10-06 scope add locales/*.json — Grouped approved homepage files plus retained task 0002 changes in the uncommitted batch; exact intended edits recorded in Context
- 2026-10-06 scope add docs/mcp.md — Grouped approved homepage files plus retained task 0002 changes in the uncommitted batch; exact intended edits recorded in Context
- 2026-10-06 advance blocked → in-progress/implement
- 2026-10-06 advance in-progress/implement → in-progress/verify
- 2026-10-06 advance in-progress/verify → in-progress/review
- 2026-10-06 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 1)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 2)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 3)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
