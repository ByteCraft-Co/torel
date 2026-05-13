#include "backend_internal.h"

#include <exception>

extern "C" TorelBackendResult torel_llvm_emit_ir(TorelBackendBuffer validated_mir) {
    try {
        const auto mir = torel::llvm_backend::decode_mir(validated_mir);
        const auto module = torel::llvm_backend::emit_module(mir);
        torel::llvm_backend::verify_llvm_module(module);
        return torel::llvm_backend::ok(module.text);
    } catch (const torel::llvm_backend::BackendError &err) {
        return torel::llvm_backend::error(err.status, err.message);
    } catch (const std::exception &err) {
        return torel::llvm_backend::error(TOREL_BACKEND_BRIDGE_FAILURE, err.what());
    } catch (...) {
        return torel::llvm_backend::error(TOREL_BACKEND_BRIDGE_FAILURE, "unknown C++ backend failure");
    }
}

extern "C" void torel_llvm_free_buffer(TorelBackendBuffer buffer) {
    delete[] buffer.data;
}
