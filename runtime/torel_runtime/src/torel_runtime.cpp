#include "torel_runtime.h"

#include <cstdlib>

extern "C" int32_t torel_exit_ok(void) {
    return 0;
}

extern "C" void torel_trap_unreachable(void) {
    std::abort();
}

extern "C" void torel_trap_overflow(void) {
    std::abort();
}

extern "C" void torel_backend_abort(const char *message) {
    (void)message;
    std::abort();
}
