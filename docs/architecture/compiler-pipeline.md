# Compiler Pipeline

Torel's compiler is organized as a sequence of explicit stages. Each stage has a crate boundary so the implementation can grow without collapsing parsing, semantic analysis, and code generation into one pile of state.

```txt
source
  -> lexer
  -> parser
  -> AST
  -> HIR
  -> name resolution
  -> typed IR
  -> type/return checks
  -> effect/failure checks
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
| HIR | `torel_ir` | Lower syntax into compiler-owned semantic structures. |
| Name resolution / typed IR | `torel_typeck` / `torel_ir` | Resolve names to stable IDs and produce typed IR. |
| Type checking | `torel_typeck` | Validate declared types, expression types, and returns. |
| Effect/failure checking | `torel_effects` | Validate declared effects and checked failure channels. |
| Ownership checking | `torel_ownership` | Enforce ownership, moves, views, and arena escape rules. |
| Code generation | `torel_codegen` | Lower checked IR toward LLVM and later MLIR/LLVM. |

## Current State

The pipeline is executable but intentionally skeletal:

- the lexer recognizes the first core tokens needed by examples
- the parser accepts a `unit` declaration, the first top-level `proc` shape, `fix` bindings, returns, paths, literals, and procedure calls
- HIR preserves unit identity and procedure structure
- type checking has built-in symbols for `Exit`, `Void`, `Bool`, `Int32`, `UInt64`, `Text`, `Never`, and `Exit.ok`
- typed IR records resolved type IDs, value IDs, proc IDs, and local IDs
- procedure symbols carry parameter and return types for call checking
- literals type as `Int32`, `Text`, and `Bool`
- `fix` bindings resolve declared types, check initializer types, and add immutable locals for later statements
- the first semantic checks reject unknown types, unknown locals, duplicate locals, unknown value paths, unknown procedure calls, non-callable values, bad argument counts, bad argument types, bare procedure values, bad local initializers, bad return types, and missing returns from non-`Void` procedures
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
