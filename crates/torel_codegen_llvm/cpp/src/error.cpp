#include "backend_internal.h"

#include <cstring>

namespace torel::llvm_backend {

namespace {

TorelBackendBuffer copy_buffer(const std::string &text) {
    auto *data = new uint8_t[text.size()];
    if (!text.empty()) {
        std::memcpy(data, text.data(), text.size());
    }
    return TorelBackendBuffer{data, text.size()};
}

} // namespace

TorelBackendResult ok(std::string output) {
    return TorelBackendResult{
        TOREL_BACKEND_OK,
        copy_buffer(output),
        TorelBackendBuffer{nullptr, 0},
    };
}

TorelBackendResult error(TorelBackendStatus status, std::string message) {
    return TorelBackendResult{
        status,
        TorelBackendBuffer{nullptr, 0},
        copy_buffer(message),
    };
}

} // namespace torel::llvm_backend
