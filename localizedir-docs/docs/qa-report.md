# QA Report — visual inspection record

Build under test: final shipped state (see `docs/decisions/0004-visual-qa.md` for the method).
Automated run: `pnpm qa` — **PASSED, exit 0 — 14 routes × 2 viewports × 2 themes (56 captures)**.
Zero console errors, zero page errors, zero HTTP ≥ 400 (all assets), zero horizontal overflow,
exactly one `<h1>` per page, no empty `<main>`, `data-theme` applied correctly, no broken images.

## Per-route results

Legend: ✔ = inspected, no defects found or defect fixed and re-verified.

| Route | desktop-light | desktop-dark | mobile-light | mobile-dark | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `/` | ✔ | ✔ | ✔ | ✔ | Hero logo sharp; primary action button uses brand amber; CardGrid paths render; code block highlighted |
| `/tutorials/getting-started/` | ✔ | ✔ | ✔ | ✔ | FileTree, pnpm-first Tabs, caution aside, ansi-colored expected output block |
| `/guides/organize-your-directories/` | ✔ | ✔ | ✔ | ✔ | FileTree renders ignored-dir comments; shared-tree example highlighted |
| `/guides/control-directory-names/` | ✔ | ✔ | ✔ | ✔ | Tip aside renders with amber background in both themes |
| `/guides/handle-unknown-locales/` | ✔ | ✔ | ✔ | ✔ | Caution aside; negotiation snippet highlighted |
| `/guides/fix-compile-errors/` | ✔ | ✔ | ✔ | ✔ | Error block styled; spans visible |
| `/reference/attribute/` | ✔ | ✔ | ✔ | ✔ | 4-column options table fits desktop; scrolls horizontally on mobile (`overflow-x: auto`, verified programmatically) |
| `/reference/localize-dir-trait/` | ✔ | ✔ | ✔ | ✔ | Longest page; right-rail TOC present with current-heading highlight; method summary table clean |
| `/reference/naming-conventions/` | ✔ | ✔ | ✔ | ✔ | Suffix + case tables clean |
| `/reference/compile-error-catalogue/` | ✔ | ✔ | ✔ | ✔ | Three diagnostic tables render; double-backtick code spans display verbatim messages |
| `/reference/include-dir-interop/` | ✔ | ✔ | ✔ | ✔ | Path-semantics code block highlighted; note aside |
| `/reference/release-notes/` | ✔ | ✔ | ✔ | ✔ | External changelog/docs.rs links render as LinkCards |
| `/explanation/how-it-works/` | ✔ | ✔ | ✔ | ✔ | `<Steps>` renders numbered list with accent markers; generated-code block |
| `/explanation/design-decisions/` | ✔ | ✔ | ✔ | ✔ | Rejected-features table; pagination "Previous → How it works" points correctly |

## Targeted checks

- **Logo sharpness (2× + 4×)** — header captured at `deviceScaleFactor: 2` in both themes, then magnified 4×: clean edges, no pixelation, no halo, no clipping; transparency reads correctly on both backgrounds. Source: 256px derivative of the repo's 1264px PNG (ADR 0002) — no upscaling anywhere in the pipeline.
- **Brand accent** — amber (`#8d570c` light / `#f9bb1f`–`#f5cd47` dark), derived from the logo's dominant `#f0c010`, verified WCAG AA at design time (`scripts/contrast.py`: 6.0:1 link-on-white light, 12.8:1 text-on-dark). Clearly not Starlight default blue; visible on links, active sidebar item, asides, focus affordances.
- **Syntax highlighting** — Shiki via Expressive Code; multiple token colors in every code block in both themes; copy button present; code scrolls within its frame (no page overflow).
- **Tables** — page-level overflow absent on all 56 captures; wide tables scroll inside `overflow-x: auto` containers (probed on `/reference/attribute/` mobile: `scrollWidth 477 > clientWidth 358`, scrollable).
- **Search (Pagefind)** — header button opens the search dialog; typing `fallback` returns **"9 results for fallback"** with page titles and excerpts (probed via Playwright, `/pagefind/*` served with no 4xx).
- **Pagination** — verified on first and last sidebar pages; labels and hrefs correct.
- **404** — `/definitely-missing/` returns HTTP 404 with the custom splash page (h1 `404`, section links); screenshot captured once per the QA method; no draft banner.
- **Dark mode** — no pure-white panels, no grey-on-grey, code background distinct from page background in all 28 dark captures.

## Defects found and fixed during QA

1. Draft banner visible on the custom 404 page (caused by a `draft: true` workaround for a route conflict). Fixed by switching to Starlight's official custom-404 recipe and allowlisting its single upstream route-priority warning (ADR 0005). Re-verified: no banner, HTTP 404, clean splash.
2. Invalid `<Aside type="important">` in two pages — build error caught at first render. Fixed to `caution`/`note` and rebuilt.
3. Escaped-backtick code spans broke MDX parsing in four files. Rewritten as double-backtick code spans showing the verbatim diagnostics. Rebuilt.

## Gate results (final state, re-run as the last action before packaging)

| Gate | Result |
| :--- | :--- |
| `pnpm build` | exit 0 — 15 pages, sitemap, Pagefind index, llms.txt/llms-full.txt generated; **1 warning line, exactly the allowlisted Starlight 404 route conflict** (`scripts/check-warnings.sh`: "no unexpected warnings") |
| `pnpm check` | exit 0 — zero errors, zero warnings (`astro sync` + `tsc --noEmit`, see ADR 0007) |
| `pnpm lint` | 0 issues in 15 files |
| `pnpm qa` | exit 0 — 14 routes × 2 viewports × 2 themes |
| Eyeball pass | complete, recorded in the table above |
