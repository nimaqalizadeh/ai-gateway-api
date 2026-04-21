# ADR 000 — Bootstrap: tooling, structure, conventions
- **Status:** Accepted
- **Date:** 2026-04-21
- **Deciders:** Nima

---

## Context

We are starting the `ai-gateway` project and need to establish a coherent, stable foundation: crate identity, Rust edition, toolchain pinning, module layout, task running, formatting, linting, and pre-commit hygiene. We also need a lightweight process for recording architectural decisions themselves. Making all of these choices explicit up front prevents drift and makes onboarding unambiguous.

---

## Decisions

### 1. Crate identity

The crate is named `ai-gateway` and targets the **Rust 2021 edition**. Edition 2024 is available but async-fn-in-trait support is still settling across the ecosystem; we will revisit when the situation stabilises.

### 2. Toolchain pin

The Rust toolchain is pinned to **1.93.1** via a `rust-toolchain.toml` at the repository root, using the `minimal` profile with `rustfmt` and `clippy` components added explicitly. Pinning eliminates "works on my machine" breakage from silent toolchain upgrades and gives us a clear MSRV baseline.

```toml
# rust-toolchain.toml
[toolchain]
channel  = "1.93.1"
profile  = "minimal"
components = ["rustfmt", "clippy"]
```

### 3. Module layout

Source files follow the **modern Rust convention**: a module `foo` is represented as `src/foo.rs` alongside `src/foo/` for submodules, never `src/foo/mod.rs`. This keeps the file tree easier to navigate in editors and avoids the proliferation of files all named `mod.rs`.

The project is a **single crate**, not a Cargo workspace. We will introduce a workspace only if a genuine need for multiple independently-versioned crates emerges.

### 4. Task runner

We use **`just`** as the task runner. `just` provides a clean, self-documenting `justfile` with no implicit rules and no Make-specific gotchas (phony targets, tab sensitivity, recursive variable expansion). Common recipes — `fmt`, `lint`, `test`, `run`, `docker-build` — live in the `justfile` at the repo root.

### 5. Formatter configuration

`rustfmt` is configured via `rustfmt.toml` with **stable-only options**. A previous attempt included `imports_granularity`, which turned out to be nightly-only and caused failures on the pinned toolchain; it was removed. Current config:

```toml
# rustfmt.toml
edition   = "2021"
max_width = 100
```

Line width is set to 100 rather than the default 80, which fits comfortably on modern screens without requiring aggressive reformatting of moderately long lines.

### 6. Linter configuration

`clippy` is configured via `clippy.toml` with the MSRV set to match the pinned toolchain:

```toml
# clippy.toml
msrv = "1.93.1"
```

Clippy is run with `-D warnings` (deny all warnings) in both the pre-commit hook and the `justfile` lint recipe:

```
cargo clippy -- -D warnings
```

This ensures lints are never silently ignored.

### 7. Pre-commit hooks

Three hooks run automatically on every commit:

| Hook | Command |
|---|---|
| Format check | `cargo fmt --check` |
| Lint (deny warnings) | `cargo clippy -- -D warnings` |
| Spell check | `typos` |

`cargo fmt --check` (rather than `cargo fmt`) is used so the hook fails visibly without silently mutating files mid-commit.

### 8. ADR adoption

We adopt **Architecture Decision Records** as the mechanism for recording significant architectural choices. This document is the first ADR and is itself the record of that adoption. ADRs use a lightweight format: status, date, context, decision(s), and consequences. They are append-only; superseded decisions are marked as such rather than deleted.

### 9. Documentation layout

All project documentation lives under **`docs/`**. ADRs specifically live under **`docs/adr/`** and are numbered sequentially with three-digit, zero-padded prefixes (`000-`, `001-`, `002-`, …). This keeps documentation co-located with the source, version-controlled, and discoverable without any external tooling.

### 10. `.gitignore` policy for secrets

`.env` and `.env.*` are ignored by default, with `!.env.example` negated back in. API keys and credentials must never be tracked; an `.env.example` template, when present, documents which environment variables the application expects without leaking values. This policy is locked in now because the gateway will handle live OpenAI and Anthropic credentials from early phases onward.

---

## Consequences

**Positive**

- The toolchain is reproducible across machines and CI without any extra setup beyond installing `rustup`.
- New contributors have a single `just --list` to discover available tasks.
- Formatting and lint disagreements are caught locally, before they reach code review.
- Architectural rationale is captured where future maintainers will find it — in the repository itself, not in Notion or a chat thread.

**Negative / trade-offs**

- Pinning the Rust toolchain means compiler upgrades are a deliberate, manual step. Security fixes and new lints do not arrive automatically; someone must bump `rust-toolchain.toml`.
- Contributors need `just` and `pre-commit` installed in addition to `rustup`. Fresh-machine setup therefore has three prerequisites, not one.
- Choosing stable-only `rustfmt` options gives up grouped/nested import formatting. Large files will have noisier `use` blocks until those options stabilise.
- Starting as a single crate keeps things simple now, but migrating to a Cargo workspace later (if multiple publishable crates emerge) will involve real effort: splitting `Cargo.toml`, moving files, and updating paths.
- Edition 2024 is deferred; any ergonomic gains there (particularly around async traits) are unavailable until we revisit this decision.

Edition 2024 and workspace adoption are deferred but not foreclosed. This ADR should be revisited when either becomes appropriate, via a superseding ADR rather than an edit.
