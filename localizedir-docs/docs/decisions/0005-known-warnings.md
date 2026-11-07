# ADR 0005: Known warnings — allowlisted build log lines

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** The zero-warning release gate (`pnpm build` must emit no warning lines).

## Allowlisted warnings

Exactly one warning line in `pnpm build` output is allowlisted. Everything else fails the gate.

### 1. Starlight's custom-404 route conflict

```text
[WARN] [build] Could not render `/404` from route `/[...slug]` as it conflicts with higher priority route `/404`.
```

**Origin:** Astro's static-route priority resolution, triggered by Starlight's own documented recipe for a custom 404 page (create `src/content/docs/404.md`). Starlight injects a dedicated `/404` route **and** its `[...slug]` route emits a path for the `404` docs entry; Astro keeps the higher-priority injected route and warns about the discarded duplicate. Every site following the official customization guide produces this line.

**Why allowlisted:**

- The warning describes a duplicate that is intentionally discarded; the rendered site is correct — `dist/404.html` is the custom page, served with HTTP 404 (verified in QA).
- The alternatives were strictly worse: without a custom 404, Starlight logs `Entry docs → 404 was not found.` (also a warning) and serves a bare fallback page; marking the entry `draft: true` silences the conflict but renders a visible "this content is a draft" banner on the production error page — a real defect instead of a cosmetic log line.
- It originates in Starlight/Astro routing internals and cannot be fixed from this project.

**Narrow filter:** `scripts/check-warnings.sh` excludes exactly the line above (full-string match on the message substring `Could not render \`/404\` from route`) with the reason in a comment; any other `[WARN]`/`warning` line fails the gate.

## Verification

The final gate sequence in `docs/qa-report.md` records the unfiltered and filtered warning counts for the shipped build.
