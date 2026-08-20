#![cfg_attr(all(target_os = "none", not(feature = "std")), no_std)]

#[cfg(all(target_os = "none", not(feature = "std")))]
use alloc::ffi::CString;
#[cfg(all(target_os = "none", not(feature = "std")))]
use alloc::format;
use anyhow::{Error, Result};
#[cfg(all(target_os = "none", not(feature = "std")))]
use core::alloc::GlobalAlloc;
use wasmtime::{Engine, Module, Store};

#[cfg(all(target_os = "none", not(feature = "std")))]
extern crate alloc;

mod c {
    #[cfg(all(target_os = "none", not(feature = "std")))]
    use core::ffi;

    unsafe extern "C" {
        pub fn printf(format: *const u8, ...);
        #[cfg(all(target_os = "none", not(feature = "std")))]
        pub fn malloc(size: usize) -> *mut u8;
        #[cfg(all(target_os = "none", not(feature = "std")))]
        pub fn free(ptr: *mut u8);
        #[cfg(all(target_os = "none", not(feature = "std")))]
        pub fn exit(status: ffi::c_int) -> !;
    }
}

#[unsafe(no_mangle)]
extern "C" fn invoke_wasmtime(ptr: *const u8, len: usize) -> i32 {
    if ptr.is_null() || len == 0 {
        return 1;
    }
    // SAFETY: The caller must provide a readable buffer of `len` bytes.
    let data = unsafe { core::slice::from_raw_parts(ptr, len) };

    match invoke_wasmtime_impl(data) {
        Ok(()) => 0,
        Err(_) => 2,
    }
}

fn invoke_wasmtime_impl(data: &[u8]) -> Result<()> {
    let engine = Engine::default();

    let module = Module::new(&engine, data).map_err(Error::msg)?;
    let mut linker = wasmtime::Linker::new(&engine);

    let mut store = Store::new(&engine, ());

    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |_a: i32, _fd: i32, _aa: i32, _ciovs: i32| {
                // let buf = first_non_empty_ciovec(memory, ciovs)?;
                unsafe {
                    c::printf(b"fd_write called!\n\0".as_ptr());
                };
                Ok(0)
            },
        )
        .map_err(Error::msg)?;
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "environ_get",
            |_a: i32, _b: i32| {
                unsafe {
                    c::printf(b"environ_get called!\n\0".as_ptr());
                };
                Ok(0)
            },
        )
        .map_err(Error::msg)?;
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "environ_sizes_get",
            |_a: i32, _b: i32| {
                unsafe {
                    c::printf(b"environ_size_get called!\n\0".as_ptr());
                };
                Ok(1)
            },
        )
        .map_err(Error::msg)?;
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "proc_exit",
            |_a: i32| -> wasmtime::Result<()> {
                unsafe {
                    c::printf(b"proc_exit called!\n\0".as_ptr());
                };
                Err(wasmtime::Error::msg("exit"))
            },
        )
        .map_err(Error::msg)?;
    linker
        .func_wrap("env", "num", |a: i32| {
            unsafe {
                c::printf(b"num called with '%d'\n\0".as_ptr(), a);
            };
        })
        .map_err(Error::msg)?;
    linker.module(&mut store, "", &module).map_err(Error::msg)?;

    let run = linker
        .get(&mut store, "", "_start")
        .map_err(Error::msg)?
        .into_func()
        .ok_or_else(|| Error::msg("module does not export a function named `_start`"))?;
    run.typed::<(), ()>(&store)
        .map_err(Error::msg)?
        .call(&mut store, ())
        .map_err(Error::msg)?;

    Ok(())
}

#[cfg(all(target_os = "none", not(feature = "std")))]
struct LibcAlloc;

#[cfg(all(target_os = "none", not(feature = "std")))]
#[global_allocator]
static GLOBAL_ALLOC: LibcAlloc = LibcAlloc;

#[cfg(all(target_os = "none", not(feature = "std")))]
unsafe impl GlobalAlloc for LibcAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let header_size = core::mem::size_of::<*mut u8>();
        let Some(size) = layout
            .size()
            .checked_add(layout.align() - 1)
            .and_then(|size| size.checked_add(header_size))
        else {
            return core::ptr::null_mut();
        };
        let raw = unsafe { c::malloc(size) };
        if raw.is_null() {
            return raw;
        }
        let start = unsafe { raw.add(header_size) };
        let offset = start.align_offset(layout.align());
        let aligned = unsafe { start.add(offset) };
        unsafe {
            aligned
                .sub(header_size)
                .cast::<*mut u8>()
                .write_unaligned(raw)
        };
        aligned
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        let header_size = core::mem::size_of::<*mut u8>();
        let raw = unsafe { ptr.sub(header_size).cast::<*mut u8>().read_unaligned() };
        unsafe { c::free(raw) };
    }
}

#[cfg(all(target_os = "none", not(feature = "std")))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        c::printf(b"\nPANIC!!!\n\0".as_ptr());
        match CString::new(format!("{}", info)) {
            Ok(message) => c::printf(b"%s\n\0".as_ptr(), message.as_ptr()),
            _ => c::printf(b"unknown error while generating panic\n\0".as_ptr()),
        }
        c::exit(1);
    }
}
