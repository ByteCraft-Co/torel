#ifndef TOREL_RUNTIME_H
#define TOREL_RUNTIME_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TOREL_RUNTIME_VERSION_MAJOR 0
#define TOREL_RUNTIME_VERSION_MINOR 1
#define TOREL_RUNTIME_VERSION_PATCH 0

int32_t torel_exit_ok(void);
void torel_trap_unreachable(void);
void torel_trap_overflow(void);
void torel_backend_abort(const char *message);

#ifdef __cplusplus
}
#endif

#endif
