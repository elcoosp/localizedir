# ADR 0004: Visual QA — Playwright screenshots of every route × 2 viewports × 2 themes, plus a manual eyeball pass

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** A green build proves nothing about what readers actually see.

## Decision

Visual QA is mandatory and blocking. `pnpm qa` (`scripts/qa-screenshots.mjs`) enumerates every `index.html` under `dist/` — no hand-maintained route list — and for each route captures desktop (1440×900) and mobile (390×844) viewports in both light and dark themes against a production preview server.

## Automated checks per capture

The script fails (exit 1) on:

- console errors and uncaught page errors
- HTTP ≥ 400 responses, including missing assets and broken images
- horizontal overflow (`scrollWidth > innerWidth + 1`)
- missing or duplicate `<h1>`
- `<main>` text shorter than 200 characters (blank page)
- theme not actually applied (`data-theme` mismatch)
- any broken `<img>`

Screenshots land in `.qa/screenshots/` (gitignored, not part of the deliverable) and failures are written to `.qa/failures.json`.

## The eyeball pass

Automated checks catch breakage, not ugliness. After every QA run, the screenshots are inspected — at minimum the home page, one page per Diátaxis section, and the longest page, in both viewports and both themes — against the checklist in the QA report (`docs/qa-report.md`): logo sharpness at 2×, brand accent on links/active states, syntax highlighting and copy buttons, tables, asides/cards/tabs/steps/file trees, sidebar states, on-page TOC, pagination, mobile nav, dark-mode contrast, typography, and search results. Outcomes are recorded per route in `docs/qa-report.md`.

## Release gate order

1. `pnpm build` — exit 0, zero warning lines
2. `pnpm check` — 0 errors, 0 warnings
3. `pnpm lint` — clean
4. `pnpm qa` — exit 0 across every route × 2 viewports × 2 themes
5. eyeball pass recorded in `docs/qa-report.md`

Any change to config, CSS, logo, theme, or content after a QA pass invalidates it; the full gate re-runs immediately before packaging the deliverable.

## Considered and not added

`@axe-core/playwright` accessibility assertions were considered. They were not added because the Starlight theme already ships WCAG-audited baseline components, the accent palette is contrast-verified at design time (see `scripts/contrast.py` and ADR 0002's palette derivation), and the manual checklist covers focus rings and contrast. Revisit if the site grows custom components beyond theme overrides.
