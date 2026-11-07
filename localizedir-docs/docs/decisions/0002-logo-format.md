# ADR 0002: Logo format — native-resolution PNG, no SVG exists in the source repository

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** Branding of the localizedir documentation site.

## Situation

The source repository ships exactly one logo asset: `docs/logo.png`, a 1264×1264 RGBA PNG (transparent background) showing nested colored folders with a globe. No SVG original exists anywhere in the repository (checked `docs/`, `src/assets/`, `public/`, `static/`, `assets/`, `images/` and the whole tree). **This is a recorded logo-quality risk**: the project's brand asset is raster-only.

## Decision

1. **Use the project's actual logo** — never invent a replacement or fall back to Starlight's default branding.
2. **Do not upscale or trace.** The native file (1264×1264) is larger than every display size by a wide margin:
   - Header logo displays at ~2rem height; the 256px derivative gives ≥ 8× headroom (sharp on any retina display).
   - The favicon uses a 64px derivative.
3. **Derive, optimize, commit** via `scripts/gen-assets.mjs` (sharp, PNG palette quantization):
   - `src/assets/logo.png` — 256×256, 12 KB, alpha preserved (used in the header and hero).
   - `public/favicon.png` — 64×64, 2.6 KB, alpha preserved (referenced by Starlight's `favicon` option).
   - The original 1264px file in the source repository stays untouched.
4. **Single asset for both themes.** The logo's transparency is real (alpha range 0–255, verified), and its palette (golden amber `#f0c010`, folder blues/greens/purples, globe blue) reads correctly on both the light and dark page backgrounds. Verified in the Phase 7 screenshot pass in both themes at 2× zoom. Starlight therefore gets one `logo.src` instead of separate `light`/`dark` variants.

## Rejected alternatives

- **Trace the PNG to SVG** — the asset is a complex 3D render with gradients and soft shadows; an automatic trace would produce a large, visually degraded vector blob rather than a brand improvement.
- **Upscale the PNG to fake a larger logo** — that is the pixelation failure mode the pipeline exists to prevent, and it is unnecessary here because the source already exceeds every display size.
- **Generate @2x/@3x variants of a small raster** — unnecessary for the same reason; the "largest rendered size as the base" rule is satisfied by the 1264px original itself.

## Consequences

- Logo sharpness is bounded by the 256px derivative's headroom, not by CSS. If the site ever needs a hero-scale logo larger than 256px, regenerate from the source repository's original at the required size (`scripts/gen-assets.mjs` is the single place to change).
- The Astro experimental `svgOptimizer` is **not** enabled: the project imports no SVGs. Revisit this decision if a vector logo ever lands upstream.
- If the upstream project ever publishes an SVG original, convert the pipeline to SVG (with SVGO, `removeViewBox: false`) and replace the derivatives.
