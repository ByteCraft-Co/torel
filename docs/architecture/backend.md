# Backend Architecture

Torel uses a hybrid compiler architecture:

```txt
Rust frontend and semantic authority
  -> Torel MIR
  -> MIR validation
  -> backend trait
  -> C++ LLVM backend / C debug backend
  -> runtime ABI
```

The backend boundary is deliberately boring. Rust owns Torel meaning. Backends own emission.

## Rust Owns

- source loading and sessions
- lexer, parser, AST, and HIR
- name resolution and typed IR
- type, return-flow, effect, failure, and ownership checks
- MIR construction
- MIR validation
- diagnostics
- CLI orchestration
- package manager, language server, formatter, and test harness later

## Backends Own

- translating validated MIR into backend-specific output
- LLVM module creation
- LLVM type lowering
- LLVM basic block and instruction emission
- LLVM verification
- object emission and target-machine integration later
- runtime calls and ABI lowering
- debug metadata and optimization passes later

Backends may reject unsupported MIR with structured errors. They must not decide whether Torel source code is semantically valid.

## Backend Trait

The Rust backend trait accepts `MirModule` and a target:

- MIR text
- C source
- LLVM IR
- object file
- executable

Every backend must validate MIR before emission or require a caller-owned validated wrapper once MIR stabilizes. Structured backend errors carry a kind, message, and optional Torel source span.

## CLI Targets

The CLI now reserves backend-facing targets:

- `--emit mir`: implemented textual MIR
- `--emit c`: reserved C debug backend
- `--emit llvm-ir`: reserved C++ LLVM backend
- `--emit object`: reserved native object output
- `--emit executable`: reserved native executable output

Reserved targets still run the Rust frontend, semantic checks, effect/failure/ownership placeholders, MIR lowering, and MIR validation before returning structured backend errors.

## Sacred Rule

LLVM and MLIR must never become the frontend semantic model. They are output details. Torel semantics live in Rust-owned IR and MIR.
