#include "backend_internal.h"

namespace torel::llvm_backend {

void verify_llvm_module(const LlvmTextModule &module) {
    if (module.text.empty()) {
        throw BackendError{TOREL_BACKEND_LLVM_VERIFICATION_FAILED, "LLVM module is empty"};
    }
}

} // namespace torel::llvm_backend
