#include "backend_internal.h"

namespace torel::llvm_backend {

MirModuleView decode_mir(TorelBackendBuffer buffer) {
    if (buffer.data == nullptr && buffer.len != 0) {
        throw BackendError{TOREL_BACKEND_INVALID_MIR, "MIR buffer pointer is null"};
    }

    return MirModuleView{
        std::string_view{reinterpret_cast<const char *>(buffer.data), buffer.len},
    };
}

} // namespace torel::llvm_backend
