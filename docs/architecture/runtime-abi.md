# Runtime ABI

The runtime starts as a tiny C ABI and grows only when the language needs it.

## Initial ABI Surface

- process exit mapping
- unreachable trap
- overflow trap
- backend panic or abort hook
- runtime version identity

## Primitive Representation

- `Exit`: initially represented as an `i32` process status contract
- `Bool`: one-byte or LLVM `i1` internally, normalized at ABI boundaries
- `Int32`: signed 32-bit integer
- `UInt64`: unsigned 64-bit integer
- `Void`: no value
- `Never`: no returning value; emitted as unreachable or trap path

## Later ABI Work

- product type layout
- choice/tagged union layout
- text and buffer representation
- allocator and arena hooks
- deterministic drop hooks
- failure return ABI
- effect representation if needed at runtime
- views and owned references
- Torel-to-C and C-to-Torel calls
- platform differences

The runtime is part of the language contract. Helpers that affect observable behavior must be specified before they become backend dependencies.
