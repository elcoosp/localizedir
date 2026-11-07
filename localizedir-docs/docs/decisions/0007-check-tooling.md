# ADR 0007: Type checking — `tsc --noEmit` over generated Astro types instead of `astro check`

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** The zero-error/zero-warning `check` gate on the build machine used to produce this site.

## Decision

`pnpm check` runs `astro sync && tsc --noEmit` (TypeScript **6.0.3**, pinned per ADR 0001) instead of `astro check`. `astro sync` generates `.astro/types.d.ts` (content collection types), and `tsc --noEmit` type-checks every TypeScript file this project owns — `astro.config.mjs`, `src/content.config.ts`, `src/env.d.ts`, and the generated types — under `astro/tsconfigs/strict`.

## Why not `astro check`

`astro check` delegates .astro-file type checking to `@astrojs/check` → `@astrojs/language-server` → `@astrojs/astro2tsx`. `astro2tsx` ships a **wasm-only** napi-rs binding (`astro2tsx.wasm32-wasi.wasm`) whose emnapi stack fails to load in the production environment with:

```text
TypeError: this.bridge.setLastError is not a function
    at NodeEnv.deleteMe (@emnapi/runtime/dist/emnapi.js:855:21)
```

This was verified to be independent of anything this project controls:

- Node **24.22.2** and Node **22.23.4** (the verified Astro 7.4 baseline) both fail identically.
- `@astrojs/astro2tsx` **0.1.1, 0.1.2, and 0.1.3** all fail identically.
- `@emnapi/runtime` **2.0.0-beta.2 and 2.0.0-beta.3** with `@napi-rs/wasm-runtime` **1.2.4 and 1.2.7** all fail identically.
- `astro check` 0.9.12's TypeScript-7 guard was bypassed correctly (TypeScript 6.0.3 installed per ADR 0001); the failure is purely the wasm-napi bridge.

## Why the substitution is equivalent here

`astro check`'s unique capability is type-checking **`.astro` component files** through astro2tsx. This project contains **zero `.astro` files** — it is a content-only Starlight site (Markdown/MDX content, one CSS file, JSON i18n, and config/`<script>`-free TS). The TypeScript surface it owns is exactly what `tsc --noEmit` checks: configuration, the content collection schema extension, and `src/env.d.ts`.

## Consequences

- The release gate is unchanged in strength: `pnpm check` must exit 0 with zero errors and zero warnings.
- `@astrojs/check` is not in `devDependencies` (it pulls the broken wasm chain). If the site ever grows custom `.astro` components, revisit this ADR — by then `astro check`/astro2tsx will likely have a fixed release; the upstream error to watch for is quoted above.
- TypeScript remains pinned to 6.0.3 per ADR 0001; `tsc` runs under that exact version.
