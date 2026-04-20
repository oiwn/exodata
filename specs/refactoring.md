# Refactoring ideas

---

## Part 2 — Murky Parts (clarify before touching)

### 2.1 `render_columns` silently drops `hostname` for system tables
**File:** `src/components/insights/common.rs:163–171`

Detects system tables by presence of `sy_name`, then drops `hostname` from display.
Intent is reasonable (avoid redundant columns) but is invisible — no comment, and the
heuristic will silently misfire if detection fails.

**Action:** either move the suppression to the server layer (don't return `hostname` in
system insight data) or add a clear comment explaining the heuristic.

---

### 2.2 `page == 0 → 1` normalization is duplicated
**Files:** `src/server/cache.rs:85` AND inside each distinct-query function

Cache key normalization already canonicalizes page 0 → 1. The data layer should not
need to repeat it.

**Action:** remove the guard from data functions once `TableResult` uses `TableCacheValue`
and normalization is the single source of truth.

---

## Part 3 — Known Dead Code (confirm, then delete)

- `src/common.rs` — backwards-compat re-export shim, likely unused
- `src/stellarhosts.rs` — unconfirmed, check then delete
- Both noted in `specs/ideas.md` doubts section

---

## Refactoring Order

| Step | What | Risk | Est. lines delta |
|------|------|------|-----------------|
| 1 | Add `InsightDef` struct + `INSIGHTS` registry in `mod.rs` | Low | +30 |
| 2 | Migrate each insight component to export `DEF` with SQL | Low | per file |
| 3 | Write generic SQL executor + `get_insight(slug)` server fn | Medium | +80 |
| 4 | Wire cache warm at startup from `INSIGHTS` | Low | +10 |
| 5 | Delete old data layer: wrappers, server fns, distinct-query fns | Low | −500 |
| 6 | Change `TableResult` to use `TableCacheValue` | Medium (all callers) | −30 |
| 7 | Fix `render_columns` hostname suppression | Low | 0 |
| 8 | Delete `src/common.rs` + verify `src/stellarhosts.rs` | Low | −N |
| 9 | Add CLI `insights` commands (`list`, `run <slug>`, `run-all`) | Low | +~100 |

Steps 1–6 eliminate ~500 lines of structural repetition. Step 9 closes the loop on
testability.
