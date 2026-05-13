# Compiler Stage Ownership

Torel keeps language meaning on the Rust side and backend machinery behind MIR.

| Stage | Owner | Notes |
| --- | --- | --- |
| Source/session | Rust | `torelc` and `torel_session`. |
| Lexer/parser | Rust | Tokens, AST, spans, syntax diagnostics. |
| HIR | Rust | Compiler-owned semantic shape. |
| Typed IR | Rust | Name resolution, stable IDs, type checking. |
| Effects/failures | Rust | Checked semantic contracts. |
| Ownership | Rust | Moves, views, arenas, escape rules. |
| MIR | Rust | Backend contract, CFG, primitive operations. |
| MIR validation | Rust | Required before any backend receives MIR. |
| Backend trait | Rust | Structured backend API and errors. |
| C++ LLVM backend | C++ | LLVM emission and verification only. |
| Runtime | C/C++ ABI | Process exit, traps, later allocation and layout hooks. |

This split keeps LLVM and target behavior out of frontend semantics.
