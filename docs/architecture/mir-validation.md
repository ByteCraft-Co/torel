# MIR Validation

MIR validation is the airlock before backend code generation. Rust constructs MIR, then Rust validates it. C++ backends receive only validated MIR.

The validator proves:

- every module contains at least one function
- every function entry block exists
- every block ID is unique inside a function
- every block has exactly one terminator
- every jump target exists
- every branch target exists
- every branch condition has type `Bool`
- every return matches the function return type
- every local reference points at a declared local or parameter
- every temporary reference points at a declared temporary
- every temporary is defined before use
- every temporary is assigned at most once
- immutable locals are assigned only during initialization
- every direct call targets a known procedure
- no structured syntax such as `break` or `continue` reaches MIR

Validation errors are compiler bugs or backend input bugs, not user-code type errors. User-code validity belongs to the parser, HIR lowering, type checker, effect checker, failure checker, and ownership checker before MIR construction.
