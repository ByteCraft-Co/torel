#ifndef TOREL_BACKEND_INTERNAL_H
#define TOREL_BACKEND_INTERNAL_H

#include "torel_llvm_backend.h"

#include <memory>
#include <string>
#include <string_view>
#include <vector>

namespace torel::llvm_backend {

struct BackendError {
    TorelBackendStatus status;
    std::string message;
};

struct MirModuleView {
    std::string_view text;
};

struct LlvmTextModule {
    std::string text;
};

TorelBackendResult ok(std::string output);
TorelBackendResult error(TorelBackendStatus status, std::string message);

MirModuleView decode_mir(TorelBackendBuffer buffer);
LlvmTextModule emit_module(const MirModuleView &module);
void emit_function();
void emit_expression();
void emit_control_flow();
void map_type();
void declare_runtime_calls();
void verify_llvm_module(const LlvmTextModule &module);

} // namespace torel::llvm_backend

#endif
