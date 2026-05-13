# LLVM Backend

LLVM is Torel's first serious native target. It must remain behind MIR.

## Phase 1: LLVM IR

- map primitive Torel types to LLVM types
- emit functions and basic blocks from MIR
- emit returns, branches, jumps, local storage, primitive operations, and direct calls
- declare runtime trap hooks
- run LLVM verification
- return verified LLVM IR

## Current Repository State

The Rust crate `torel_codegen_llvm` validates MIR and exposes structured backend errors. Its `cpp/` directory contains the C++ bridge boundary and subsystem layout for LLVM emission, but the bridge is intentionally not wired into Cargo until LLVM and a C++ compiler are available in the build environment.

Today:

- `--emit mir` is implemented
- `--emit llvm-ir` validates MIR, then fails honestly with a structured unsupported-target error
- `--emit object` and `--emit executable` are reserved behind verified LLVM IR
- C++ bridge stubs define the ownership/error boundary and engine-room subsystems

The first future LLVM implementation must replace the reserved error with C++ emission plus LLVM verification.

## Phase 2: Target Configuration

- target triple
- data layout
- target machine
- platform-aware object emission

## Phase 3: Linking

- link object files with the Torel runtime
- produce native executables
- make process exit and trap hooks work end to end

## Phase 4: Expansion

- broader primitive ABI
- multiple functions and exported visibility
- product and choice layouts
- runtime calls

## Phase 5: Hardening

- reproducible output
- optimization levels
- sanitizer compatibility
- debug metadata
- cross-target strategy

Checked integer arithmetic must not silently lower to unchecked LLVM arithmetic. Runtime overflow traps or LLVM overflow intrinsics are required before checked arithmetic is considered supported by the LLVM backend.
