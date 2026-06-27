## Rust — Koda conventions

- Errors: `anyhow` for binaries, `thiserror` for libraries
- Async: tokio only — no mixed runtimes
- Unwrap: forbidden outside tests — use `?` or `unwrap_or_else`
- Logging: `tracing` with structured fields, never `println!` in production
- Types: prefer newtypes for domain IDs (avoid bare `String`)
- Lifetime: minimize explicit lifetimes; prefer owned types in async contexts
- Clippy: treat all warnings as errors (`#![deny(clippy::all)]` in libs)
- Tests: unit tests in the same file with `#[cfg(test)]`, integration tests in `tests/`
- Format: `cargo fmt` enforced, 4-space indent
