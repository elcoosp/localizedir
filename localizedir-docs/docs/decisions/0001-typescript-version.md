# ADR 0001: Pin TypeScript to 6.0.3 for `astro check`

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** Astro 7.4.7 / @astrojs/starlight 0.42.10 documentation build.

## Decision

`typescript` is pinned to the latest 6.x release (**6.0.3**, exact, in `devDependencies`) so that `astro check` runs on a supported TypeScript version. TypeScript 7.0.2 is the current `latest` on npm and is **not** used.

## Rationale

As of Astro 7.4, `astro check` does not support TypeScript 7. Running it against TypeScript 7 fails with:

```text
astro check does not currently support TypeScript 7.0
```

An experimental workaround (`tsc` plus `@astrojs/ts-content-mapper`) exists, but it is not the stable default and is not required for this project. The official guidance is to remain on TypeScript 6.x until `astro check` gains TypeScript 7 support.

## Consequences

- `pnpm check` runs against TypeScript 6.0.3 and reports 0 errors / 0 warnings.
- The exact pin (`6.0.3`, no caret) prevents an accidental upgrade to the 7.x line via `pnpm up --latest`.
- When Astro ships TypeScript 7 support in `astro check`, bump the pin and delete this ADR's constraint.
