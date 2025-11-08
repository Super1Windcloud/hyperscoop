# Repository Guidelines

## Project Structure & Module Organization
- `src/` hosts the `hp` CLI entrypoint (`main.rs`) and orchestrates high-level flows, while reusable logic lives in `crates/command`, `crates/shim`, `crates/hash`, and `crates/hscoop_macro`.
- Assets and docs live in `resources/`, `img/`, and `hyperscoop_source_bucket/` (Scoop manifests). Automation scripts stay in `script/`, and release helpers reside in `justfile`.
- Examples (`examples/`) and benches (`benches/hash_benchmark.rs`) are compiled separately; keep crate-specific fixtures near their modules to simplify review.

## Build, Test, and Development Commands
- `cargo build --workspace --all-features` compiles every crate; run `cargo build --release` before distributing binaries.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -D warnings` enforce formatting and lint compliance.
- `cargo test --workspace --all-features` executes unit/integration suites; `cargo bench --bench hash` triggers the Criterion benchmark.
- `just release` drives the `cargo br` release pipeline, `just cross` builds the Windows/MSVC matrix, and `just update_hash` refreshes bucket metadata plus pushes remotes.
- Use `cargo run --example pg_bar` for quick progress-bar regressions without touching the main CLI.

## Coding Style & Naming Conventions
- Default to Rust 2021 with 4-space indents and `rustfmt` formatting; keep modules single-purpose and favor dependency injection over global state.
- Use `snake_case` for functions/modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants; CLI verbs should mirror Scoop terminology (`hp install`, `hp bucket`).
- Avoid `unsafe` unless essential; workspace lints treat `unsafe_code` and `unreachable_pub` as warnings, so annotate any justified exceptions inline.

## Testing Guidelines
- Co-locate unit tests inside each module under `#[cfg(test)]`; async flows should use `#[tokio::test(flavor = "multi_thread")]` to match runtime defaults.
- Integration tests that touch real buckets belong in `tests/` or `examples/`; mock filesystem paths under `resources/testdata` to keep runs deterministic.
- Maintain coverage for CLI parsing, manifest hashing, and shim generation before triggering release jobs; refresh Criterion baselines when performance-critical code changes.

## Commit & Pull Request Guidelines
- Follow the existing log style: optional emoji prefix (e.g., `:panda_face:`) plus a concise imperative summary (`:panda_face: update hash flow`).
- PRs should describe the change, impacted crates, and include sample `hp` output or screenshots when altering UX; link issues and note any `just` routines that were run.
- Confirm formatting, tests, and relevant `just` scripts in the PR checklist; bucket changes must mention the updated manifests under `hyperscoop_source_bucket/`.
