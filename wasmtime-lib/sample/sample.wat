(module
  ;; Imported host functions provided by `invoke_wasmtime` in wasmtime-lib.
  (import "env" "num" (func $num (param i32)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "environ_get" (func $environ_get (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "environ_sizes_get" (func $environ_sizes_get (param i32 i32) (result i32)))

  (memory (export "memory") 1)

  ;; Compute 0 + 1 + ... + 9 = 45 in a loop, then call `env.num` with the
  ;; result and with 6 * 7 = 42.
  (func (export "_start")
    (local $i i32)
    (local $sum i32)
    (local.set $i (i32.const 0))
    (local.set $sum (i32.const 0))
    (block $done
      (loop $loop
        (br_if $done (i32.ge_u (local.get $i) (i32.const 10)))
        (local.set $sum (i32.add (local.get $sum) (local.get $i)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)))
    (call $num (local.get $sum))
    (call $num (i32.mul (i32.const 6) (i32.const 7))))
)