# Roadmap

Committed follow-up work. The active generation task remains in
[ctx.md](ctx.md); uncommitted possibilities live in [ideas.md](ideas.md).

## Prose Out of Git — Upload Channel (Round A2, deferred)

Generated prose is deployment data, not source: nothing under
`content/systems/` is tracked (already untracked and gitignored; prompts
and the guide at `content/` root stay tracked). Serve prose through the
same pattern as the Parquet data: upload + bind mount.

- [ ] Dockerfile: remove `COPY content ./content` (builder) and
      `COPY --from=builder /app/content/systems /app/content/systems`
      (runtime). Nothing in the web build reads content at compile time,
      and CI checkouts no longer contain the directory.
- [ ] `deploy.yml`: add volume `"{{ content_path }}:/app/content/systems:ro"`
      beside the data mount.
- [ ] `group_vars/all.yml`: `content_path: /app/content` (host), and
      `local_content_path: "{{ playbook_dir }}/../../../content/systems"`
      (mirrors `local_data_path`).
- [ ] New `playbooks/upload-content.yml`: ensure
      `{{ content_path }}/{packs,systems}` exist; localhost pack of
      `*/description.md` only (`notes.toml` stays local, never uploaded)
      as `content-<date>-<HHMM>-<N>sys.tar.gz` (fail on zero
      descriptions); upload; unarchive into `{{ content_path }}/systems/`;
      prune packs keeping the newest 5; `docker restart {{ app_name }}`
      plus the same HTTP health-wait task as `deploy.yml`.

Deferred 2026-09-11 until Round B content generation improves (see
[ctx.md](ctx.md)); deployment is the last stage.
- [ ] Justfile `ansible-upload-content` recipe; DEPLOY.md section
      covering the content channel and the rollout ordering rule:
      upload first (files stage harmlessly), deploy second (mount
      activates on container recreate). Old images keep their baked prose
      until the deploy.
- [ ] Verify: `ansible-playbook --syntax-check`, local `tar tzf` sanity,
      live upload + deploy + curl one described and one undescribed host
      page. No Rust changes required (`EXO_CONTENT_DIR` default already
      resolves to the mount target; the loader tolerates a missing
      directory).

## Update tailwind

```
Command [tailwindcss] requested version v4.2.1, but a newer version v4.3.3 is available, you can try it out by setting the LEPTOS_TAILWIND_VERSION=v4.3.3 env var and re-running the command
```

## Shared Measurement Selection — Required Before Release

- [x] Define source-row selection: stellar hosts use the row with the most
  populated summary measurements; planets use the unique `default_flag = 1`
  row. No medians, nearest-median selection, or cross-row filling. See
  [detail contracts](web-backend.md#detail-contracts) for scoring and ties.
- [ ] Implement and verify shared selection for detail pages and exports,
  retaining raw records and a selected-record index for references, errors,
  and qualifiers. Confirm hero values, summary cards, and comparisons agree.
- [x] Reuse shared selection when implementing description-generation inputs.
- [ ] Present measurement qualifiers consistently: distinguish mass, minimum
  mass, and upper/lower bounds using the selected source record. This remains
  a separate pre-release requirement from selecting existing numeric values.

## Localized Route Declarations

- [ ] Define shared page routes once rather than repeating them for English,
  Simplified Chinese, and Japanese. Preserve existing URLs, parameters, query
  state, locale synchronization, lazy loading, SSR modes, 404 behavior, and the
  single app-wide `<main>`. Verify direct loads and client navigation across
  supported locales. Adding a locale should not require copying the route list.

## Separate Graphics Page

- [ ] Specify a dedicated page for graphical catalog exploration, distinct
  from homepage distribution bars (issue references #10 and #11). Define its
  content and interactions before choosing the rendering library.

^^^ no there must be separate page with graphics i think we'll use "rust-ui"
&&& Preserve the requirement for a separate graphics page. Homepage bars do not complete it; "rust-ui" remains a candidate until the page requirements and library choice are evaluated.

## Issue Review

- [ ] Review issues with the user one at a time against their current bodies,
  current code, and any necessary production evidence. Check GitHub state
  when that review is requested; do not reuse historical issue counts or
  closure recommendations as current facts.
- [ ] Clarify remaining acceptance criteria for harness integrations (#131,
  #133) and the CLI package description (#111), accounting for the existing
  OpenCode workflow and hosted MCP service.
- [ ] Investigate externally observed issues only with a reproduction:
  unexpected agent injection (#97), production caching/render delays (#87),
  and disputed dataset interpretations (#72, #90).
- [ ] Evaluate the infrastructure alternative raised in #43 as a separate
  research task, not as an assumed defect in the deployed proxy.

Issue numbers identify prior requests, not verified current open/closed status.
Remaining concrete product and maintenance ideas are grouped in
[ideas.md](ideas.md). Do not comment on, edit, or close GitHub issues without
the user's authorization.
