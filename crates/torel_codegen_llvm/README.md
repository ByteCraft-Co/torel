# Torel LLVM Backend

This crate is the Rust-facing shell for the future C++ LLVM backend.

Current state:

- Rust validates MIR before backend emission.
- `LlvmBackend` exposes structured backend errors.
- `--emit llvm-ir`, `--emit object`, and `--emit executable` are reserved CLI targets.
- C++ bridge headers and subsystem stubs live under `cpp/`.

The local development machine must provide LLVM and a C++ compiler before the bridge can emit and verify LLVM IR.
