# Object Emission And Linking

Object and executable output come after verified LLVM IR.

## Object Phase

- configure target triple and data layout
- create target machine
- emit object files from verified LLVM IR
- report structured backend errors for target failures

## Linking Phase

- link Torel object files with the runtime
- choose platform linker strategy
- preserve runtime trap symbols
- map `Exit` to process status
- keep debug and sanitizer paths available for later

Until this phase exists, `--emit object` and `--emit executable` should fail honestly.
