# Repository Guidelines

## Project Structure & Module Organization
- Keep crate-specific fixtures near their modules to simplify review.

## Build, Test, and Development Commands
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` enforce formatting and lint compliance.
- `cargo test --workspace --all-features` executes unit/integration suites; `cargo bench --bench hash` triggers the Criterion benchmark.
- `just release` drives the `cargo br` release pipeline, `just cross` builds the Windows/MSVC matrix, and `just update_hash` refreshes bucket metadata plus pushes remotes.
- Use `cargo run --example multi` for quick progress-bar regressions without touching the main CLI.

## Coding Style & Naming Conventions
- Keep modules single-purpose and favor dependency injection over global state.
- CLI verbs should mirror Scoop terminology (`hp install`, `hp bucket`).
- Avoid `unsafe` unless essential; annotate any justified exceptions inline.

## Testing Guidelines
- Co-locate unit tests inside each module under `#[cfg(test)]`; async flows should use `#[tokio::test(flavor = "multi_thread")]` to match runtime defaults.
- Integration tests that touch real buckets belong in `tests/` or `examples/`; mock filesystem paths under a temporary directory to keep runs deterministic.
- Maintain coverage for CLI parsing, manifest hashing, and shim generation before triggering release jobs; refresh Criterion baselines when performance-critical code changes.

## Commit & Pull Request Guidelines
- Follow the existing log style: optional emoji prefix (e.g., `:panda_face:`) plus a concise imperative summary (`:panda_face: update hash flow`).
- PRs should describe the change, impacted crates, and include sample `hp` output or screenshots when altering UX; link issues and note any `just` routines that were run.
- Confirm formatting, tests, and relevant `just` scripts in the PR checklist; bucket changes must mention the updated manifests under `hyperscoop_source_bucket/`.
