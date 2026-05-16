use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::host::HermesHost;

pub struct PluginState {
    pub wasi: WasiCtx,
    pub host: HermesHost,
}

pub struct WasmPlugin {
    pub name: String,
    module: Module,
}

pub struct PluginManager {
    engine: Engine,
    linker: Linker<PluginState>,
    plugins: Mutex<std::collections::HashMap<String, WasmPlugin>>,
}

impl PluginManager {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        
        // Link WASI features
        wasmtime_wasi::add_to_linker(&mut linker, |state: &mut PluginState| &mut state.wasi)?;
        
        // Link Hermes specific host functions
        linker.func_wrap(
            "hermes", 
            "log", 
            |_caller: Caller<'_, PluginState>, msg_ptr: i32, msg_len: i32| {
                // Host function to let Wasm log via tracing
                debug!("Wasm plugin logged message at ptr {}, len {}", msg_ptr, msg_len);
                Ok(())
            }
        )?;

        Ok(Self {
            engine,
            linker,
            plugins: Mutex::new(std::collections::HashMap::new()),
        })
    }

    pub async fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<String, anyhow::Error> {
        let path_ref = path.as_ref();
        let name = path_ref.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        info!("Loading Wasm plugin: {}", name);
        let module = Module::from_file(&self.engine, path_ref)?;
        
        let plugin = WasmPlugin {
            name: name.clone(),
            module,
        };

        self.plugins.lock().await.insert(name.clone(), plugin);
        Ok(name)
    }

    pub async fn execute_plugin(&self, name: &str, func_name: &str) -> Result<(), anyhow::Error> {
        let plugins = self.plugins.lock().await;
        let plugin = plugins.get(name).ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();

        let state = PluginState {
            wasi,
            host: HermesHost::new(),
        };

        let mut store = Store::new(&self.engine, state);
        
        // Add fuel to prevent infinite loops in untrusted Wasm
        store.set_fuel(10_000_000)?;

        let instance = self.linker.instantiate_async(&mut store, &plugin.module).await?;
        
        let func = instance.get_typed_func::<(), ()>(&mut store, func_name)?;
        
        // Call the exported function
        func.call_async(&mut store, ()).await?;
        
        Ok(())
    }
}
