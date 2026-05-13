# C++ Engine Room

The C++ backend exists because LLVM's native API and target machinery are C++-first. C++ is the backend engine room, not the Torel courtroom.

## Subsystems

- MIR decoder or bridge adapter
- module emitter
- function emitter
- type mapper
- local storage manager
- expression emitter
- control-flow emitter
- call emitter
- runtime call emitter
- backend error builder
- LLVM verifier
- target configuration
- object emitter later
- debug metadata emitter later
- optimization pipeline later

No single C++ file should become the whole backend. Each subsystem should map to a stable backend responsibility.

## First Achievement

The first C++ achievement is verified LLVM IR from validated MIR. It is not native linking, whole-program optimization, or full ABI lowering.
