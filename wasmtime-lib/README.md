# wasmtime-lib

A `#![no_std]` static library build of Wasmtime for bare-metal (freestanding)
targets. It exposes a single C ABI entry point:

```c
int invoke_wasmtime(const uint8_t *data, size_t len);
```

which loads a WebAssembly module from a readable in-memory buffer of `len`
bytes and runs its `_start` export. It returns zero on success, 1 for an invalid buffer, and 2
when compilation, instantiation, or execution fails. Imports from
`wasi_snapshot_preview1` and `env` are stubbed out and print via `printf`.

The crate builds as a freestanding static library on targets without std
(where `target_os == "none"`); it builds as a normal std library everywhere
else. `std` is also available as an optional, additive feature
(`--features std`).

## Building for aarch64-unknown-none

From the repository root, enter the development shell. The shell supplies Rust
1.95, the bare-metal target, WABT, QEMU, and an AArch64 C cross-compiler.
Direnv users can run `direnv allow` instead.

```
nix develop
cargo build -p wasmtime-lib --target aarch64-unknown-none
```

The freestanding configuration is detected automatically, so
`--no-default-features` is not required (but is also fine to pass). The
resulting archive is at:

```
target/aarch64-unknown-none/debug/libwasmtime_lib.a
```

## Testing

Compile the sample C driver together with the library, and run the result
under user-mode qemu with an aarch64 Linux sysroot.

Run the sample from the repository root inside the development shell:

```
./wasmtime-lib/run-sample.sh
```

The script rebuilds the archive, compiles `sample/sample.wat`, cross-compiles
the C driver, and executes it with `qemu-aarch64`.

Expected output:

```
Hello from the wasmtime-lib sample!
num called with '45'
num called with '42'
wasmtime invocation completed
```

`sample.c` also implements the platform hooks the freestanding build needs
(see `../docs/examples-minimal.md`): `wasmtime_tls_get`/`wasmtime_tls_set` and,
because `custom-virtual-memory` is enabled, the `wasmtime_mmap_*`/`mprotect`/
`page_size` functions.