# Rust/C++ Backend Boundary

The Rust/C++ boundary is a hard contract, not an implementation detail.

## Ownership Rules

- Rust creates MIR.
- Rust validates MIR.
- Rust serializes or bridges validated MIR into C++.
- C++ returns LLVM IR, object bytes, executable bytes, or structured backend errors.
- No Rust panic crosses into C++.
- No C++ exception crosses into Rust.
- Every buffer has one owner.
- Every returned string or object has a documented destruction path.
- The boundary is versioned once MIR stabilizes.

## Error Rules

Backend errors are structured:

- invalid MIR
- unsupported target
- unsupported feature
- bridge failure
- LLVM verification failure

Normal user-facing diagnostics should be rendered by Rust. C++ may attach MIR node or source-span references, and Rust translates those into Torel diagnostics where possible.

## Design Direction

The first C++ bridge should be small: load validated MIR, emit target-independent LLVM IR, run LLVM verification, and return either verified IR or a structured backend error. Object files, linking, optimization levels, and debug metadata come after that.
