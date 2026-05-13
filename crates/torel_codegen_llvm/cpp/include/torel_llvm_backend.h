#ifndef TOREL_LLVM_BACKEND_H
#define TOREL_LLVM_BACKEND_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum TorelBackendStatus {
    TOREL_BACKEND_OK = 0,
    TOREL_BACKEND_UNSUPPORTED_FEATURE = 1,
    TOREL_BACKEND_INVALID_MIR = 2,
    TOREL_BACKEND_LLVM_VERIFICATION_FAILED = 3,
    TOREL_BACKEND_BRIDGE_FAILURE = 4
} TorelBackendStatus;

typedef struct TorelBackendBuffer {
    const uint8_t *data;
    size_t len;
} TorelBackendBuffer;

typedef struct TorelBackendResult {
    TorelBackendStatus status;
    TorelBackendBuffer output;
    TorelBackendBuffer error_message;
} TorelBackendResult;

TorelBackendResult torel_llvm_emit_ir(TorelBackendBuffer validated_mir);
void torel_llvm_free_buffer(TorelBackendBuffer buffer);

#ifdef __cplusplus
}
#endif

#endif
