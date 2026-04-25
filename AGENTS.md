# AGENTS.md

Agent-facing instructions for working in `/home/anson/src/protoglot`.

## Overall Philosophy & Directives

1. Develop your opinion carefully and then defend it. Do not simply agree with the user if you feel that a different course of action is better. Explain your reasoning, how you are quantifying better, and argue until you are persuaded otherwise or the user definitively chooses a different path.
2. You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` and avoid falling back to text-only tools like `rg` or `grep` unless a plain-text search is explicitly requested.
3. When troubleshooting or debugging, do not speculate and make code changes based on speculation. Do the real debugging needed to find the issue in code, or add necessary logging/debugging to isolate it.
4. When asked to review code, focus only on bugs, issues, code smell, poor maintainability, performance issues, and safety problems. Do not discuss what is good unless explicitly requested. Start with the assumption that there are concrete issues to find, and take the full product scope into consideration.
5. When asked to write an RFC/PRD, start with no assumptions other than what the user explicitly says. Ask questions until the scope is understood, keep the process interactive until sign-off, and challenge product or architectural premises when warranted.

## Scope And Priorities

1. Keep changes minimal, local, and consistent with existing patterns.
2. Prefer `mise` tasks over raw commands whenever possible.
3. Protoglot is a high-performance CLI data generator and absorber. Treat throughput, low overhead, and graceful behavior under network/data errors as top-level product requirements.
4. Optimize CLI UX for common workflows: sensible defaults, discoverable flags, clear status, and actionable errors.
5. Be selective with dependencies. Get explicit permission before adding any new dependency, and check the current crate version before proposing one.
6. Always prevent or fix clippy/rustfmt issues; do not suppress lints to hide problems.
7. If customer-visible behavior changes, update `CHANGELOG.md` under `[Unreleased]`.

## Delivery Philosophy

- Default to complete, production-target implementations in a single pass when scope is clear.
- Do not artificially split work into incremental phases based on outdated assumptions about delivery speed.
- Use phased rollout/checkpointing for operational safety and validation, not as a substitute for complete implementation.
- If constraints force sequencing, make that explicit and keep each step fully correct.
- Prefer user-native CLI workflows over internal convenience. If two designs are functionally equivalent, choose the one that reduces setup burden and cognitive load.
- When writing plans or discussing scope/timing, assume the agent is doing the implementation work unless explicitly told otherwise.

## Repository Layout

- Rust binary crate root: `Cargo.toml`
- Application entry point: `src/main.rs`
- Configuration and CLI parsing: `src/config/`
- Emit path: `src/emitter.rs`, `src/generators/`, `src/transports/`
- Absorb path: `src/absorber/`
- Docker packaging: `Dockerfile`, `entrypoint.sh`
- Product and workflow notes: `conductor/`
- Mise task definitions: `.mise/config.toml`
- CI workflow: `.github/workflows/merge-and-pr-build.yml`

## Core Commands

Check available tasks first:

```bash
mise tasks
```

Build:

```bash
mise run build
mise run build:release
cargo build
cargo build --release
```

Tests:

```bash
mise run test
mise run tv
mise run testes
mise run test:ci
cargo nextest run
cargo test --verbose
```

Use `cargo nextest run` for normal Rust test runs when available. The GitHub Actions workflow currently runs `cargo test --verbose`, so use that when matching CI behavior is important or `nextest` is unavailable.

Single test/filter:

```bash
cargo nextest run <test_name_or_filter>
cargo test <test_name_or_filter>
```

Lint and formatting:

```bash
mise run clippy
mise run fmt
mise run fmt:check
cargo clippy --all-targets --all-features
cargo fmt -- --config-path .rustfmt.stable.toml
```

Rustfmt has unstable preferences in `.rustfmt.toml`. If using a nightly toolchain and you need to apply the full project formatting style, run:

```bash
cargo +nightly fmt
```

Run the binary locally:

```bash
mise run help
mise run help:absorber
mise run config
cargo run -- --help
cargo run -- absorber --help
cargo run -- config
```

Docker smoke check:

```bash
mise run docker:smoke
docker build -t protoglot:local .
docker run --rm protoglot:local --help
```

## Required Project Rules

- Write a reproduction test before fixing a bug or adding a feature whenever feasible.
- Verify every task with the smallest relevant tests first, then broader checks as appropriate.
- Do not stage or commit changes unless explicitly asked.
- Do not stage or commit files under `conductor/` or other Conductor-generated artifacts unless the user explicitly overrides this rule.
- Preserve config file compatibility unless the user explicitly accepts a breaking change.
- Keep command-line flags and config keys aligned. Config structs use serde `camelCase` in several places.
- Do not introduce blocking work on Tokio async paths except behind `spawn_blocking` or an equivalent existing pattern.
- Avoid panics, `unwrap`, and `expect` in production paths for recoverable runtime failures. Convert expected failures into actionable errors.

## Rust Style Guide

- Edition: Rust `2024`.
- Prefer simple, idiomatic Rust and existing module patterns over broad refactors.
- Naming: `PascalCase` for types/traits, `snake_case` for functions/modules/variables, `SCREAMING_SNAKE_CASE` for constants.
- Keep imports explicit and minimal; preserve existing alias conventions like `Parser as _`.
- Use doc comments on public structs, enums, functions, and modules when adding or changing public API.
- Error handling: `anyhow` is currently used at application boundaries and for broad runtime errors. For new domain-level APIs, prefer typed errors when callers can reasonably recover or assert specific failure modes.
- Logging: use `log`/`env_logger`; keep messages technical, precise, and actionable. Avoid noisy logs in hot paths.
- Async/networking: follow existing Tokio, Hyper, Reqwest, and Rustls patterns. Prefer graceful handling of peer disconnects and malformed input over task crashes.
- Performance: avoid unnecessary allocations in event generation, transport send loops, decompression paths, and absorber hot paths.

## Testing Style

- Existing tests use `pretty_assertions`, `sealed_test`, `test-log`, and `tokio::test`.
- Prefer adding tests near the code under test unless an integration-style test is more useful.
- For configuration tests that touch user config paths, use `sealed_test` or another isolation strategy consistent with existing tests.
- For network tests, use ephemeral ports or existing helper patterns; avoid hard-coded ports that can conflict on shared machines.
- Keep tests deterministic. Avoid sleeps unless they are unavoidable and bounded.

## Product Notes

- Protoglot emits and absorbs data across TCP, UDP, HTTP, HTTPS, and TCPS paths.
- Supported message shapes include NDJSON, Syslog 3164, Syslog 5424, and Syslog 5424 octet-count-related paths.
- The absorber reports real-time stats including event counts and raw/decompressed byte counts.
- Compression/decompression support includes gzip, zstd, lz4, brotli, and snappy paths.
- TLS and certificate behavior is user-visible; changes must be validated carefully and documented in help text or changelog entries as needed.

## Verification Checklist Before Finishing

1. Ran the smallest relevant tests first.
2. Used `mise` tasks for common checks when available.
3. Ran `mise run test`, `cargo nextest run`, or `cargo test --verbose` for broader Rust verification when appropriate.
4. Ran `mise run clippy` or `cargo clippy --all-targets --all-features` for non-trivial Rust changes when feasible.
5. Ran formatting with the project rustfmt setup if code was edited.
6. Updated `CHANGELOG.md` if customer-visible behavior changed.
7. Kept output consistent with performance, CLI UX, error-handling, and async constraints.

## Useful Paths

- Product definition: `conductor/product.md`
- Product guidelines: `conductor/product-guidelines.md`
- Development workflow: `conductor/workflow.md`
- General style guide: `conductor/code_styleguides/general.md`
- Formatting config: `.rustfmt.toml`, `.rustfmt.stable.toml`
- Mise task definitions: `.mise/config.toml`
- CI workflow: `.github/workflows/merge-and-pr-build.yml`
