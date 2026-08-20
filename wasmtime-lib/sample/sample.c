/*
 * Sample C program that links against the `wasmtime-lib` static library
 * (built for a bare-metal target such as `aarch64-unknown-none`) and drives
 * it through the exported `invoke_wasmtime` function.
 *
 * It also provides the minimal platform hooks that the no_std wasmtime build
 * requires (see `docs/examples-minimal.md`). The implementations here are
 * backed by Linux/glibc so the resulting binary can run under user-mode qemu,
 * e.g.:
 *
 *   aarch64-unknown-linux-gnu-gcc sample.c libwasmtime_lib.a -o sample
 *   qemu-aarch64 -L <sysroot> ./sample sample.wasm
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>

#include "wasmtime-lib.h"

/* ------------------------------------------------------------------ */
/* Minimal platform hooks required by the no_std wasmtime build.       */
/* ------------------------------------------------------------------ */

#define WASMTIME_PROT_READ (1u << 0)
#define WASMTIME_PROT_WRITE (1u << 1)
#define WASMTIME_PROT_EXEC (1u << 2)

struct wasmtime_memory_image;

uint8_t *wasmtime_alloc(size_t size) { return malloc(size); }

void wasmtime_free(uint8_t *ptr) { free(ptr); }

void wasmtime_log(const char *message) { fputs(message, stdout); }

_Noreturn void wasmtime_panic_handler(const char *message) {
  fprintf(stderr, "\nPANIC!!!\n%s\n", message);
  exit(1);
}

static int wasmtime_to_mmap_prot(uint32_t prot_flags) {
  int flags = 0;
  if (prot_flags & WASMTIME_PROT_READ)
    flags |= PROT_READ;
  if (prot_flags & WASMTIME_PROT_WRITE)
    flags |= PROT_WRITE;
  if (prot_flags & WASMTIME_PROT_EXEC)
    flags |= PROT_EXEC;
  return flags;
}

int wasmtime_mmap_new(uintptr_t size, uint32_t prot_flags, uint8_t **ret) {
  void *rc = mmap(NULL, size, wasmtime_to_mmap_prot(prot_flags),
                  MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  if (rc == MAP_FAILED)
    return 1;
  *ret = rc;
  return 0;
}

int wasmtime_mmap_remap(uint8_t *addr, uintptr_t size, uint32_t prot_flags) {
  void *rc = mmap(addr, size, wasmtime_to_mmap_prot(prot_flags),
                  MAP_FIXED | MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  if (rc == MAP_FAILED)
    return 1;
  return 0;
}

int wasmtime_munmap(uint8_t *ptr, uintptr_t size) {
  return munmap(ptr, size) != 0;
}

int wasmtime_mprotect(uint8_t *ptr, uintptr_t size, uint32_t prot_flags) {
  return mprotect(ptr, size, wasmtime_to_mmap_prot(prot_flags)) != 0;
}

uintptr_t wasmtime_page_size(void) { return (uintptr_t)sysconf(_SC_PAGESIZE); }

int wasmtime_memory_image_new(const uint8_t *ptr, uintptr_t len,
                              struct wasmtime_memory_image **ret) {
  (void)ptr;
  (void)len;
  *ret = NULL;
  return 0;
}

int wasmtime_memory_image_map_at(struct wasmtime_memory_image *image,
                                 uint8_t *addr, uintptr_t len) {
  /* Unreachable because wasmtime_memory_image_new always returns NULL. */
  (void)image;
  (void)addr;
  (void)len;
  abort();
}

void wasmtime_memory_image_free(struct wasmtime_memory_image *image) {
  /* Unreachable because wasmtime_memory_image_new always returns NULL. */
  (void)image;
  abort();
}

static __thread void *wasmtime_tls_slots[4];

uint8_t *wasmtime_tls_get(size_t slot) { return wasmtime_tls_slots[slot]; }

void wasmtime_tls_set(size_t slot, uint8_t *ptr) {
  wasmtime_tls_slots[slot] = ptr;
}

/* ------------------------------------------------------------------ */

int main(int argc, char **argv) {
  printf("Hello from the wasmtime-lib sample!\n");

  if (argc != 2) {
    fprintf(stderr, "usage: %s <module.wasm>\n", argv[0]);
    return 1;
  }

  FILE *f = fopen(argv[1], "rb");
  if (!f) {
    perror("fopen");
    return 1;
  }
  fseek(f, 0, SEEK_END);
  long size = ftell(f);
  fseek(f, 0, SEEK_SET);

  uint8_t *buf = malloc((size_t)size);
  if (!buf) {
    fprintf(stderr, "malloc failed\n");
    return 1;
  }
  if (fread(buf, 1, (size_t)size, f) != (size_t)size) {
    fprintf(stderr, "fread failed\n");
    return 1;
  }
  fclose(f);

  int rc = invoke_wasmtime(buf, (size_t)size);

  free(buf);
  if (rc != 0) {
    fprintf(stderr, "wasmtime invocation failed: %d\n", rc);
    return 1;
  }
  printf("wasmtime invocation completed\n");
  return 0;
}
