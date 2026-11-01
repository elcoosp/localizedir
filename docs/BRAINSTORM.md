# Brainstorm: localizedir v0.2.0

> Outcome of the 2026-11 overhaul review. Every item marked **Shipped** is
> implemented in this release; items marked **Rejected** were deliberately
> left out, with the reason — so future contributors do not re-litigate them
> blindly.

## 1. Performance

| # | Idea | Status | Notes |
|---|------|--------|-------|
| 1.1 | **Embed the base directory once** instead of one `include_dir!` per (variant × locale) pair. One macro expansion and one static tree; lookups navigate the embedded tree with `Dir::get_dir`. | Shipped | Generated code shrinks from `V × L` `include_dir!` expansions to exactly 1. Faster compiles on large template trees, single source of truth, and runtime fallback becomes a cheap tree navigation instead of a generated match arm. |
| 1.2 | **Zero-filesystem work at runtime.** Locales are discovered once at compile time and emitted as a `&'static [&'static str]` slice. | Shipped | Was already true for the locale list; kept and now documented. |
| 1.3 | **No allocations on the lookup hot path beyond one `PathBuf` join.** | Shipped | `get_dir` compares full embedded paths, so one `PathBuf` join is the minimum needed; `dir_name()`/`parent_folder()` are `&'static str`. |
| 1.4 | Runtime locale *normalization* (case-folding `FR` → `fr`, `-` → `_`). | Rejected | Needs an allocation per lookup, and silently guessing what the user meant hides typos. Exact match against the discovered names is predictable; the `fallback` option covers the operational need. |
| 1.5 | Manual tree descent (component-by-component `file_name()` matching) to avoid the `PathBuf` join. | Rejected | Through a trait object the borrows stop being `'static`; the returned reference would borrow from a local. One small allocation buys obvious correctness. |

## 2. Modularity

| # | Idea | Status | Notes |
|---|------|--------|-------|
| 2.1 | Split the single 280-line `lib.rs` of `localizedir-macros` into focused modules: `args` (attribute parsing), `naming` (case + suffix inference), `discovery` (filesystem scanning and path resolution), `expand` (codegen). | Shipped | The proc-macro entry point is now a 4-line facade; everything else is a pure, unit-testable function. |
| 2.2 | Make the codegen path return `syn::Result<TokenStream2>` and convert errors with `Error::into_compile_error`. | Shipped | Errors now point at the offending token (the attribute, the variant, the case value) instead of the generic *proc-macro panicked* message. |
| 2.3 | Workspace-level shared metadata (`[workspace.package]`) and dependency table (`[workspace.dependencies]`). | Shipped | One place to bump versions; the two crates can never drift. |

## 3. DRY

| # | Idea | Status | Notes |
|---|------|--------|-------|
| 3.1 | Delete the hand-rolled `pascal_to_snake_case` (it mangled acronyms: `APIResponseDirs` → `a_p_i_response`) and use the `convert_case` crate that was *already a dependency but never imported*. | Shipped | `APIResponseDirs` → `api_response`, `MJMLTemplates` → `mjml`. |
| 3.2 | Implement the suffix stripping the README promised (`Templates`, `Documents`, `Assets`, …) in one table-driven place, ordered longest-first. | Shipped | Also fixes the README/code drift where only `Dirs`/`Dir` were stripped. |
| 3.3 | Single lint configuration in `[workspace.lints]` (rustc `missing_docs`, clippy `pedantic`) inherited by both crates. | Shipped | CI only needs `-D warnings`. |
| 3.4 | Reuse `include_dir`'s own `$VAR` interpolation semantics for macro-time validation instead of inventing a second path grammar. | Shipped | A path like `$CARGO_MANIFEST_DIR/locales` validates exactly the way `include_dir!` will resolve it. |

## 4. KISS

| # | Idea | Status | Notes |
|---|------|--------|-------|
| 4.1 | `Option`-first lookup: `try_to_localized_dir` returns `Option<&'static Dir<'static>>`; `to_localized_dir` stays as the panicking convenience wrapper. | Shipped | Rust-idiomatic; the panicking variant documents its `# Panics` section. |
| 4.2 | Fallback locale expressed as one trait method (`fallback_locale() -> Option<&'static str>`, default `None`) + a small generated override only when `fallback = "..."` is configured. | Shipped | No fallback machinery exists unless the user asks for it. |
| 4.3 | Reject degenerate inputs up front (non-enum targets, fielded/generic/empty enums, duplicate resulting directory names) with precise compile errors. | Shipped | Better than emitting code that fails deep inside generated `match` arms. |
| 4.4 | Keep the locale model as plain `&str` keys. | Shipped | A newtype/`LanguageTag` type belongs to higher-level i18n crates. |

## 5. Complementary features

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| 5.1 | `try_to_localized_dir` / `to_localized_dir` split (see 4.1). | Shipped | |
| 5.2 | `#[localizedir("path", fallback = "en")]` — fall back to another locale's directory when the requested locale is absent. | Shipped | Falls back only at the panicking API; `try_…` stays pure. |
| 5.3 | `#[localizedir("path", parent = "custom")]` — override folder inference. | Shipped | |
| 5.4 | `#[localizedir("path", case = "snake" \| "kebab" \| "flat")]` — directory naming scheme for variants. | Shipped | |
| 5.5 | Per-variant rename: `#[dir_name = "welcome_v2"]` on an enum variant. | Shipped | |
| 5.6 | New trait methods fixing README drift: `base_path()`, `full_path(locale)`, `has_locale(locale)`, `fallback_locale()`. | Shipped | The old README documented `base_path`/`full_path` that did not exist. |
| 5.7 | Embedded-file access preserved: the embedded `Dir` is the real `include_dir::Dir`, so `get_file`, `files()`, etc. all work on returned directories. | Shipped | Covered by integration tests reading fixture files. |
| 5.8 | Runtime locale negotiation (BCP-47 prefix matching, `en-GB` → `en`). | Rejected | A whole i18n subdomain; compose with the `fallback` option or a `LocaleSet` type later if demand appears. |
| 5.9 | Multiple base paths per enum / glob-based file filtering. | Rejected | YAGNI; `include_dir` 0.7 has no filter hook without forking it. |
| 5.10 | serde support / generated locale enums. | Rejected | The locale list is already exposed as `available_locales()`; wiring it into user code is a one-liner on their side. |

## 6. Quality infrastructure

| # | Item | Status |
|---|------|--------|
| 6.1 | Fix the broken doctest in `localizedir-macros` (it referenced the unlinked `localizedir` crate *and* a non-existent directory). | Shipped — examples moved to the facade crate where the fixture tree exists. |
| 6.2 | 100 % line coverage via `cargo-llvm-cov`, enforced in CI. | Shipped |
| 6.3 | `trybuild` UI tests for the compile-error surface (non-enum, fielded variant, generic enum, non-literal attribute, unknown option). | Shipped |
| 6.4 | Integration tests against a committed fixture tree, covering every trait method, fallback, overrides, and embedded content. | Shipped |
| 6.5 | Dependency refresh: `syn` 2 → **3** (major), `convert_case` 0.9 → **0.12** (major), latest `quote`/`proc-macro2`/`include_dir`; dev-deps `trybuild`, `tempfile` added. | Shipped |
| 6.6 | MSRV pinned (`rust-version = "1.88"`, edition 2024 + trybuild) and checked in CI. | Shipped |
| 6.7 | CI: fmt, clippy (pedantic, warnings denied), tests + doctests on Linux/macOS/Windows, coverage gate, `cargo-deny` (advisories/licenses/bans). | Shipped |
| 6.8 | Release automation: `git-cliff` changelog workflow, tag-triggered release workflow publishing both crates to crates.io and creating the GitHub release with changelog notes. | Shipped |
| 6.9 | Dependabot for Cargo + GitHub Actions, weekly. | Shipped |
| 6.10 | Missing MIT license file (the README badge claimed MIT). | Shipped |
