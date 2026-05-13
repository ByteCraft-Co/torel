# Torel

Torel is an experimental production-grade language project for a strict native systems/backend language with explicit memory, checked effects, checked failures, and LLVM-based code generation.

The current design source of truth lives in [docs/specs/TOREL_Language_Spec_v0.2.md](docs/specs/TOREL_Language_Spec_v0.2.md).

## Current status

Torel is early but executable. Today, the compiler can lex and parse the first core language shapes, lower them through AST and HIR, perform initial type and return-flow checks, render source-labelled diagnostics, and run check-only compiler passes through the CLI examples.

Implemented so far:

- `unit` declarations and the first top-level `proc` shape
- `fix` and `slot` bindings, assignments, returns, final block expressions, literals, paths, calls, and unary/binary operators
- `if`/`else`, `while`, unconditional `loop`, `break`, and `continue`
- built-in checking for `Exit`, `Void`, `Bool`, `Int32`, `UInt64`, `Text`, `Never`, and `Exit.ok`
- early semantic diagnostics for unknown names/types, bad calls, bad assignments, non-`Bool` conditions, missing returns, unreachable statements, and return-type mismatches

Planned next:

- fuller language coverage from the spec, including product types, choice types, contracts, generics, checked effects, checked failures, ownership, views, arenas, and unsafe/FFI boundaries
- real LLVM IR generation beyond the current check-only backend boundary
- a standard library, package/manifest workflow, and production-grade tooling around tests, benches, docs, and diagnostics

## Workspace

```txt
crates/torel              main user-facing CLI
crates/torelc             compiler driver
crates/torel_ast          syntax tree definitions
crates/torel_codegen      code generation boundary
crates/torel_diagnostics  diagnostics primitives
crates/torel_effects      checked effects and checked failures
crates/torel_ir           HIR and typed IR
crates/torel_lexer        lexer
crates/torel_ownership    ownership, move, view, and arena checks
crates/torel_parse        parser
crates/torel_session      compiler session/configuration
crates/torel_typeck       type checking
```

The intended compiler pipeline is documented in [docs/architecture/compiler-pipeline.md](docs/architecture/compiler-pipeline.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
