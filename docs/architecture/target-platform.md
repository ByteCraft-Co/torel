# Target And Platform Strategy

Torel starts target-independent and becomes target-aware gradually.

## Early Rules

- MIR is target-independent.
- ABI docs define Torel value representation before backend lowering.
- LLVM target configuration is backend-owned.
- Platform differences are explicit backend decisions.

## Later Questions

- default target triple
- cross-compilation support
- object format support
- linker selection
- standard runtime packaging
- sanitizer and debug metadata compatibility
- reproducible builds

Target behavior must not leak into parser, HIR, type checking, ownership, effects, or diagnostics.
