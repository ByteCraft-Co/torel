# Torel MIR

Torel MIR is the backend contract between the Rust-owned semantic compiler and any native backend. It is intentionally lower than typed IR and intentionally higher than LLVM.

```txt
typed IR
  -> MIR construction
  -> MIR validation
  -> backend bridge
  -> LLVM IR / C debug output / object output
```

## Responsibilities

MIR represents:

- functions with resolved procedure IDs
- parameters, locals, temporaries, and their resolved type IDs
- explicit basic blocks
- assignments to locals or temporaries
- primitive rvalues
- direct procedure calls by `ProcId`
- branches, jumps, returns, and unreachable terminators
- source spans on functions, locals, temps, statements, blocks, and terminators

MIR does not represent:

- parser tokens
- AST syntax
- HIR conveniences
- unresolved names
- `break` or `continue`
- `if`, `while`, or `loop` as high-level syntax
- LLVM instructions or target-machine details

## Control Flow

Every MIR function is a control-flow graph. A function has one entry block, every block has exactly one terminator, and all control flow is explicit through `jump`, `branch`, `return`, or `unreachable`.

Structured control flow is removed before MIR reaches a backend:

- `if` lowers to a conditional branch plus join block when needed
- `while` lowers to condition, body, and exit blocks
- unconditional `loop` lowers to body and exit blocks
- `break` lowers to a jump to the loop exit block
- `continue` lowers to a jump to the loop continuation block

## Current Lowering Scope

The current MIR lowerer covers the typed IR subset already accepted by the compiler:

- procedures and parameters
- local immutable and mutable bindings
- assignments
- explicit returns and final-expression returns
- direct calls
- `Exit.ok`, integer, text, and boolean literals
- unary and binary typed operators
- `if` / `else`
- `while`
- unconditional `loop`
- `break` and `continue` as loop jumps

Native backends may still reject features they do not support, such as `Text` layout, but MIR can carry the checked program shape for inspection and future backend lowering.

## Type Contract

MIR carries the type IDs produced by the Rust semantic pipeline. Backends consume those IDs and display names, but do not decide Torel language validity. A backend may reject unsupported MIR features, but it must not reinterpret language rules.

## Diagnostics

MIR keeps source spans so backend and validation failures can be translated back into Torel source diagnostics. Backend debug output may mention MIR or LLVM internals, but normal diagnostics should point to Torel source whenever a source span exists.
