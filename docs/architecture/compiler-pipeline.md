# Compiler Pipeline

Torel's compiler is organized as a sequence of explicit stages. Each stage has a crate boundary so the implementation can grow without collapsing parsing, semantic analysis, and code generation into one pile of state.

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

## Stage Ownership

| Stage | Crate | Responsibility |
| --- | --- | --- |
| Source loading | `torelc` / `torel_session` | Read inputs and configure a compilation session. |
| Lexer | `torel_lexer` | Convert source text into tokens with spans. |
| Parser | `torel_parse` | Convert tokens into the surface AST. |
| AST | `torel_ast` | Represent source syntax close to what the user wrote. |
| HIR / typed IR | `torel_ir` | Lower syntax into compiler-owned semantic structures. |
| Type checking | `torel_typeck` | Resolve and validate types. |
| Effect/failure checking | `torel_effects` | Validate declared effects and checked failure channels. |
| Ownership checking | `torel_ownership` | Enforce ownership, moves, views, and arena escape rules. |
| Code generation | `torel_codegen` | Lower checked IR toward LLVM and later MLIR/LLVM. |

## Current State

The pipeline is executable but intentionally skeletal:

- the lexer recognizes the first core tokens needed by examples
- the parser currently accepts a `unit` declaration and leaves item parsing for the next pass
- HIR and typed IR preserve unit identity and item counts
- type/effect/failure/ownership stages return reports
- codegen supports a check-only summary and reserves LLVM IR for the next backend phase

This lets `torelc` exercise the final shape from day one:

```powershell
cargo run -p torelc -- examples\hello.torel --emit check
cargo run -p torelc -- examples\hello.torel --emit tokens
cargo run -p torelc -- examples\hello.torel --emit ast
cargo run -p torelc -- examples\hello.torel --emit hir
```

## Design Rules

- Diagnostics should preserve source spans as far as possible.
- The AST should stay close to surface syntax.
- HIR and typed IR should be compiler-owned and free to normalize syntax.
- MLIR/LLVM must not become the frontend semantic model.
- Each semantic pass should be testable without running native codegen.
