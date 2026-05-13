#include "backend_internal.h"

namespace torel::llvm_backend {

LlvmTextModule emit_module(const MirModuleView &module) {
    (void)module;
    throw BackendError{
        TOREL_BACKEND_UNSUPPORTED_FEATURE,
        "LLVM module emission is scaffolded; MIR decoding is present but instruction lowering is not implemented",
    };
}

} // namespace torel::llvm_backend
