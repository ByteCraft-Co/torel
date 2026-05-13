# Torel

Torel is an experimental production-grade language project for a strict native systems/backend language with explicit memory, checked effects, checked failures, and LLVM-based code generation.

The current design source of truth lives in [docs/specs/TOREL_Language_Spec_v0.2.md](docs/specs/TOREL_Language_Spec_v0.2.md).

## Workspace

```txt
crates/torel              main user-facing CLI
crates/torelc             compiler driver
crates/torel_ast          syntax tree definitions
crates/torel_diagnostics  diagnostics primitives
crates/torel_lexer        lexer
crates/torel_parse        parser
crates/torel_session      compiler session/configuration
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
