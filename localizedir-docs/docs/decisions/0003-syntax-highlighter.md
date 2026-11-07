# ADR 0003: Syntax highlighting — Shiki via Starlight's built-in Expressive Code

- **Status:** Accepted
- **Date:** 2026-11-08
- **Context:** Code-heavy documentation for a Rust crate (rust, toml, bash, text, ansi fences).

## Decision

Ship with **Shiki**, the default and supported highlighter of Starlight's Expressive Code integration. No custom highlighter is added, and **Rangi is not introduced**.

## Rationale

Expressive Code — which Starlight uses for every fenced code block — highlights with Shiki at build time: zero client-side JavaScript, VS Code TextMate grammars, dual light/dark themes wired into Starlight's theme toggle, and 180+ languages including `rust`, `toml`, `bash`, and `ansi` used across this site.

Rangi's own documentation positions it as a highlighter for chat UIs, small SSG sites, and apps (~46 languages, ~13 kB all-in). It is not integrated with Expressive Code: adopting it would mean writing and maintaining a custom highlighter plugin with no upstream support, while losing transformers, Twoslash, and the language coverage a documentation site needs. This site is the exact product Shiki is designated for.

| | Rangi | Shiki |
| :--- | :--- | :--- |
| Bundle size | Tiny (~13 kB all-in) | Larger (TextMate grammars) |
| Lexing | Regex tokenizer | TextMate grammars |
| Theming | Built-in themes / pairs | Dual themes + CSS vars |
| Transformers / Twoslash | No | Yes |
| Language coverage | ~46 languages | 180+ |
| Best for | Chat UIs, SSG, small apps | Docs sites, rich code samples |

## Configuration

None. The default Starlight Shiki setup already covers this site's languages and dual-theme needs; per the configuration guidance, Expressive Code is customized only for a concrete need (non-default theme, build performance, or a language that must be guaranteed present), and none applies here.

## Consequences

- Every fenced block in the content declares a language; unlabeled fences render as plain text and are caught by `markdownlint` rule MD040.
- Terminal output uses the `ansi` tag; rendered ANSI colors appear in the tutorial's expected-output block.
- Line highlighting (`{1,3-5}`), word wrap, and the built-in copy button are available to authors with no extra tooling.
