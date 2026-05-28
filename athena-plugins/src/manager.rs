use std::path::Path;
use tokio::sync::Mutex;
use tracing::{debug, info};
use wasmtime::*;
use wasmtime_wasi::{WasiCtxBuilder, WasiP1Ctx};

use crate::host::AthenaHost;

pub struct PluginState {
    pub wasi: WasiP1Ctx,
    pub host: AthenaHost,
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
        wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |state: &mut PluginState| &mut state.wasi)?;

        // Link Athena specific host functions
        linker.func_wrap(
            "athena",
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
        let wasi_p1 = WasiP1Ctx::new(wasi);

        let state = PluginState {
            wasi: wasi_p1,
            host: AthenaHost::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_load_plugin_failure() {
        let manager = PluginManager::new().unwrap();
        let result = manager.load_plugin("/path/that/does/not/exist.wasm").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_and_execute_plugin() {
        let manager = PluginManager::new().unwrap();

        let wat_content = r#"
        (module
            (func (export "test_func"))
        )
        "#;

        let temp_file = env::temp_dir().join("test_plugin.wat");
        fs::write(&temp_file, wat_content).unwrap();

        // Load the plugin
        let name = manager.load_plugin(&temp_file).await.unwrap();
        assert_eq!(name, "test_plugin");

        // Execute the exported function
        let exec_res = manager.execute_plugin(&name, "test_func").await;
        assert!(exec_res.is_ok());

        // Execute a non-existent function
        let exec_err = manager.execute_plugin(&name, "non_existent_func").await;
        assert!(exec_err.is_err());

        // Execute on a non-existent plugin
        let exec_err2 = manager.execute_plugin("unknown_plugin", "test_func").await;
        assert!(exec_err2.is_err());
    }

    #[tokio::test]
    async fn test_load_and_execute_real_rust_plugin() {
        // Compile the plugin first using cargo (if target exists)
        // We ignore the error if wasm32-wasip1 isn't installed in the test environment,
        // but if it works, we test the actual execution.
        let status = std::process::Command::new("cargo")
            .args(&["build", "--target", "wasm32-wasip1", "--release"])
            .current_dir("../apps/example-plugin")
            .status();

        if let Ok(st) = status {
            if st.success() {
                let wasm_path = Path::new("../apps/example-plugin/target/wasm32-wasip1/release/example_plugin.wasm");
                if wasm_path.exists() {
                    let manager = PluginManager::new().unwrap();
                    let name = manager.load_plugin(wasm_path).await.unwrap();
                    assert_eq!(name, "example_plugin");

                    let exec_res = manager.execute_plugin(&name, "test_func").await;
                    assert!(exec_res.is_ok());
                }
            }
        }
    }
}

// Rust guideline compliant 2026-02-21
