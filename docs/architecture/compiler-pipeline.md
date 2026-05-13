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
| Diagnostics | `torel_diagnostics` / `torel_session` | Preserve source spans, map bytes to line/column locations, and render source-labelled errors. |
| Effect/failure checking | `torel_effects` | Validate declared effects and checked failure channels. |
| Ownership checking | `torel_ownership` | Enforce ownership, moves, views, and arena escape rules. |
| Code generation | `torel_codegen` | Lower checked IR toward LLVM and later MLIR/LLVM. |

## Current State

The pipeline is executable but intentionally skeletal:

- the lexer recognizes the first core tokens needed by examples
- the parser accepts a `unit` declaration, the first top-level `proc` shape, `fix`/`slot` bindings, assignment statements, `if`/`else` statements, `while` loops, unconditional `loop` blocks, `break`/`continue`, returns, final block expressions, paths, literals, procedure calls, parenthesized expressions, and unary/binary operators with Pratt-parsed precedence
- AST, HIR, and typed IR preserve source spans for diagnostic reporting
- HIR preserves unit identity and procedure structure
- type checking has built-in symbols for `Exit`, `Void`, `Bool`, `Int32`, `UInt64`, `Text`, `Never`, and `Exit.ok`
- typed IR records resolved type IDs, value IDs, proc IDs, and local IDs
- procedure symbols carry parameter and return types for call checking
- literals type as `Int32`, `Text`, and `Bool`
- unary and binary operators type-check strictly for `Int32` arithmetic/comparison, same-type equality, and `Bool` logic
- typed operator IR records checked integer overflow intent for arithmetic operations
- `fix` and `slot` bindings resolve declared types, check initializer types, and add locals for later statements
- assignments resolve mutable local targets and check assigned expression types
- `if`/`else` statements require `Bool` conditions, type-check branch blocks, keep branch-local bindings scoped to their branch, and participate in guaranteed-return analysis
- `while` conditions require `Bool`, loop bodies get scoped locals, `break`/`continue` require an enclosing loop, and unconditional `loop` participates in flow analysis
- unreachable statements after terminating control flow are rejected with source-labelled diagnostics
- block tail expressions type-check as procedure results and let a block complete with a value without an explicit `return`
- the first semantic checks reject unknown types, unknown locals, duplicate locals, assignment to immutable locals, invalid assignment targets, bad assignment values, non-`Bool` conditions, unknown value paths, unknown procedure calls, non-callable values, bad argument counts, bad argument types, bare procedure values, bad local initializers, bad explicit and final-expression return types, and missing guaranteed returns from non-`Void` procedures
- parser and type-checking errors render with file, line, column, source text, and an underline label
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
