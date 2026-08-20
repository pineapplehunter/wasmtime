#ifndef WASMTIME_LIB_H
#define WASMTIME_LIB_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Entry point exported by the Rust static library. */
int invoke_wasmtime(const uint8_t *data, size_t len);

/* Platform hooks consumed by the Rust static library. */
uint8_t *wasmtime_alloc(size_t size);
void wasmtime_free(uint8_t *ptr);
void wasmtime_log(const char *message);
_Noreturn void wasmtime_panic_handler(const char *message);

#ifdef __cplusplus
}
#endif

#endif
