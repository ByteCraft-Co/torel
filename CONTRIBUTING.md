# Contributing to Torel

Torel is at the compiler-foundation stage. The most valuable contributions are small, well-tested changes that strengthen the language pipeline without locking the project into premature abstractions.

## Development Setup

Install Rust with `rustup`. The repository pins the toolchain in `rust-toolchain.toml`.

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Project Shape

The compiler pipeline is intentionally split by responsibility:

```txt
source
  -> lexer
  -> parser
  -> AST
  -> HIR/typed IR
  -> type/effect/failure checks
  -> ownership checks
  -> codegen
```

See `docs/architecture/compiler-pipeline.md` before changing crate boundaries.

## Pull Requests

- Keep changes focused.
- Include tests or fixtures for language behavior changes.
- Update the spec or architecture docs when a change alters language semantics.
- Run the full local check suite before opening a PR.

## Licensing

Unless explicitly stated otherwise, contributions are accepted under the repository's dual license: MIT OR Apache-2.0.
