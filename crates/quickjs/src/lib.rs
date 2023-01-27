use anyhow::{anyhow, Result};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use wasi_common::WasiCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

pub struct QuickJS {
    engine: Engine,
    module: Module,
}

impl Default for QuickJS {
    fn default() -> Self {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, include_bytes!("../../../quickjs.wasm")).unwrap();
        Self { engine, module }
    }
}

impl TryFrom<PathBuf> for QuickJS {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path)?;
        Ok(Self { engine, module })
    }
}

impl QuickJS {
    pub fn try_execute(
        &self,
        script: &str,
        data: Option<&str>,
        inherit_stdout: bool,
        inherit_stderr: bool,
    ) -> Result<Option<String>> {
        let input = script.as_bytes().to_vec();
        let input_size = input.len() as i32;
        let data = data
            .map(|data| data.as_bytes().to_vec())
            .unwrap_or_default();
        let data_size = data.len() as i32;
        let output = Arc::new(Mutex::new(None));

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let mut wasi_ctx_builder = WasiCtxBuilder::new();
        if inherit_stdout {
            wasi_ctx_builder = wasi_ctx_builder.inherit_stdout();
        };
        if inherit_stderr {
            wasi_ctx_builder = wasi_ctx_builder.inherit_stderr();
        };

        let wasi = wasi_ctx_builder.build();
        let mut store = Store::new(&self.engine, wasi);
        let memory_type = MemoryType::new(1, None);
        Memory::new(&mut store, memory_type)?;

        linker.func_wrap(
            "host",
            "get_input_size",
            move |_: Caller<'_, WasiCtx>| -> Result<i32> { Ok(input_size) },
        )?;

        linker.func_wrap(
            "host",
            "get_input",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32| -> Result<()> {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return Err(anyhow!("failed to find host memory")),
                };
                let offset = ptr as u32 as usize;
                Ok(memory.write(&mut caller, offset, &input)?)
            },
        )?;

        linker.func_wrap(
            "host",
            "get_data_size",
            move |_: Caller<'_, WasiCtx>| -> Result<i32> { Ok(data_size) },
        )?;

        linker.func_wrap(
            "host",
            "get_data",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32| -> Result<()> {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return Err(anyhow!("failed to find host memory")),
                };
                let offset = ptr as u32 as usize;
                Ok(memory.write(&mut caller, offset, &data)?)
            },
        )?;

        let output_clone = output.clone();
        linker.func_wrap(
            "host",
            "set_output",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32, capacity: i32| -> Result<()> {
                let mut output = output_clone.lock().unwrap();

                *output = if capacity == 0 {
                    None
                } else {
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(memory)) => memory,
                        _ => return Err(anyhow!("failed to find host memory")),
                    };
                    let offset = ptr as u32 as usize;
                    let mut buffer: Vec<u8> = vec![0; capacity as usize];
                    memory.read(&caller, offset, &mut buffer)?;
                    Some(String::from_utf8(buffer)?)
                };

                Ok(())
            },
        )?;

        linker.module(&mut store, "", &self.module)?;

        // call the default function i.e. main()
        linker
            .get_default(&mut store, "")?
            .typed::<(), ()>(&store)?
            .call(&mut store, ())?;

        let output = output.lock().unwrap();
        Ok(output.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_execute() {
        let quickjs = QuickJS::default();

        let script = r#"
            'quickjs' + 'wasm'
        "#;

        let result = quickjs.try_execute(script, None, false, false).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));
    }

    #[test]
    fn try_execute_data() {
        let quickjs = QuickJS::default();

        let script = r#"
            'quickjs' + data.input
        "#;

        let data = r#"{"input": "wasm"}"#;

        let result = quickjs
            .try_execute(script, Some(data), false, false)
            .unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));
    }
}
