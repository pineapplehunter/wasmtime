use alloc::string::ToString;
use anyhow::Result;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Instance, Store};

fn run_wasi(wasi_component: &[u8]) -> Result<()> {
    let mut config = Config::default();
    config.enable_async(true);
    let engine = Engine::new(&config)?;
    let component = match deserialize(&engine, wasi_component)? {
        Some(c) => c,
        None => return Ok(()),
    };

    todo!()
}

fn deserialize(engine: &Engine, component: &[u8]) -> Result<Option<Component>> {
    match unsafe { Component::deserialize(engine, component) } {
        Ok(component) => Ok(Some(component)),
        Err(e) => {
            // Currently if custom signals/virtual memory are disabled then this
            // example is expected to fail to load since loading native code
            // requires virtual memory. In the future this will go away as when
            // signals-based-traps is disabled then that means that the
            // interpreter should be used which should work here.
            if !cfg!(feature = "custom")
                && e.to_string()
                    .contains("requires virtual memory to be enabled")
            {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}
