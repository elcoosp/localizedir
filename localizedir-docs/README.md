# localizedir documentation

Standalone [Astro Starlight](https://starlight.astro.build) documentation site for [elcoosp/localizedir](https://github.com/elcoosp/localizedir) — compile-time localized directory inclusion for Rust.

## Develop

```bash
pnpm install
pnpm dev
```

## Quality gates

```bash
pnpm build   # astro build + llms.txt generation — must exit 0 with zero warnings
pnpm check   # astro check — 0 errors, 0 warnings
pnpm lint    # markdownlint-cli2 over src/content/docs
pnpm qa      # Playwright screenshots of every route (needs pnpm preview on :4321)
```

Architecture, branding decisions, and QA records live in [`docs/`](https://github.com/elcoosp/localizedir/blob/main/docs/AGENTS.md). Content is organized by the Diátaxis framework: `tutorials/`, `guides/`, `reference/`, `explanation/`.
