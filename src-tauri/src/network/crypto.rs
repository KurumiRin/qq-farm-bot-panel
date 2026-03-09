use wasmer::{imports, Instance, Module, Store, Value};

/// WASM-based encryption for game protocol messages.
/// Wraps tsdk.wasm which provides in-place buffer encryption.
pub struct CryptoWasm {
    store: std::sync::Mutex<Store>,
    instance: Instance,
}

impl CryptoWasm {
    pub fn new() -> Result<Self, String> {
        let wasm_bytes = include_bytes!("../../tsdk.wasm");
        let mut store = Store::default();

        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| format!("Failed to compile WASM module: {}", e))?;

        // tsdk.wasm expects import object "a" with stub functions a..u
        let import_object = imports! {
            "a" => {
                "a" => wasmer::Function::new_typed(&mut store, || {}),
                "b" => wasmer::Function::new_typed(&mut store, || {}),
                "c" => wasmer::Function::new_typed(&mut store, || {}),
                "d" => wasmer::Function::new_typed(&mut store, || {}),
                "e" => wasmer::Function::new_typed(&mut store, || {}),
                "f" => wasmer::Function::new_typed(&mut store, || {}),
                "g" => wasmer::Function::new_typed(&mut store, || {}),
                "h" => wasmer::Function::new_typed(&mut store, || {}),
                "i" => wasmer::Function::new_typed(&mut store, || {}),
                "j" => wasmer::Function::new_typed(&mut store, || {}),
                "k" => wasmer::Function::new_typed(&mut store, || {}),
                "l" => wasmer::Function::new_typed(&mut store, || {}),
                "m" => wasmer::Function::new_typed(&mut store, || {}),
                "n" => wasmer::Function::new_typed(&mut store, || {}),
                "o" => wasmer::Function::new_typed(&mut store, || {}),
                "p" => wasmer::Function::new_typed(&mut store, || {}),
                "q" => wasmer::Function::new_typed(&mut store, || {}),
                "r" => wasmer::Function::new_typed(&mut store, || {}),
                "s" => wasmer::Function::new_typed(&mut store, || {}),
                "t" => wasmer::Function::new_typed(&mut store, || {}),
                "u" => wasmer::Function::new_typed(&mut store, || {}),
            }
        };

        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|e| format!("Failed to instantiate WASM: {}", e))?;

        // Call init_runtime (export "E")
        if let Ok(init_fn) = instance.exports.get_function("E") {
            let _ = init_fn.call(&mut store, &[]);
        }

        Ok(Self {
            store: std::sync::Mutex::new(store),
            instance,
        })
    }

    /// Encrypt a protobuf body buffer before sending to game server.
    pub fn encrypt_buffer(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut store = self.store.lock().unwrap();

        let create_buf = self.instance.exports.get_function("z")
            .map_err(|e| format!("No create_buf export: {}", e))?;
        let destroy_buf = self.instance.exports.get_function("A")
            .map_err(|e| format!("No destroy_buf export: {}", e))?;
        let encrypt = self.instance.exports.get_function("J")
            .map_err(|e| format!("No encrypt export: {}", e))?;
        let memory = self.instance.exports.get_memory("v")
            .map_err(|e| format!("No memory export: {}", e))?;

        let len = data.len() as i32;

        // Allocate buffer in WASM memory
        let ptr_val = create_buf.call(&mut *store, &[Value::I32(len)])
            .map_err(|e| format!("create_buf failed: {}", e))?;
        let ptr = ptr_val[0].i32().ok_or("create_buf didn't return i32")? as usize;

        // Copy data into WASM memory
        let mem_view = memory.view(&*store);
        mem_view.write(ptr as u64, data)
            .map_err(|e| format!("Memory write failed: {}", e))?;

        // Encrypt in-place
        encrypt.call(&mut *store, &[Value::I32(ptr as i32), Value::I32(len)])
            .map_err(|e| format!("encrypt failed: {}", e))?;

        // Read encrypted data back
        let mem_view = memory.view(&*store);
        let mut result = vec![0u8; data.len()];
        mem_view.read(ptr as u64, &mut result)
            .map_err(|e| format!("Memory read failed: {}", e))?;

        // Free WASM buffer
        let _ = destroy_buf.call(&mut *store, &[Value::I32(ptr as i32)]);

        Ok(result)
    }
}
