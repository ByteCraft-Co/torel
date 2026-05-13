# Unsupported Backend Features

Unsupported backend features must fail honestly. Fake code generation is worse than no code generation.

## Not Supported In The First Native Backend

- `Text` runtime layout
- strings and escapes
- arrays, slices, and buffers
- product types and choice types
- contracts
- generics
- checked failures
- checked effects
- ownership lowering
- arenas
- destructors
- async and actors
- FFI
- full standard library
- package registry

## Backend Error Shape

Backends return structured errors with:

- kind
- message
- optional source span

Rust renders user-facing diagnostics. Verbose/debug modes may expose MIR or LLVM details.
