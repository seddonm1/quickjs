use anyhow::{anyhow, bail, Result};
use std::{
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use wasi_common::WasiCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

pub struct QuickJS {
    engine: Engine,
    module: Module,
    inherit_stdout: bool,
    inherit_stderr: bool,
    memory_limit: usize,
    time_limit: Option<TimeLimit>,
}

/// A time limit to prevent long executions.
///
/// This can be extremely expensive so only use if absolutely required.
#[derive(Clone, Debug)]
pub struct TimeLimit {
    /// the maximum duration to wait for the execution to finish
    pub time_limit: Duration,
    /// the interval between evaluations whether execution is finished
    /// a more frequent evaluation will decrease the performance of the execution
    pub evaluation_interval: Duration,
}

impl QuickJS {
    /// try to instantiate a new QuickJS engine
    ///
    /// parameters:
    /// - `path`: optional override for the quickjs.wasm instance
    /// - `inherit_stdout`: route `console.log` calls to stdout
    /// - `inherit_stderr`:route `console.error` calls to stdout
    /// - `memory_limit`: optional memory limit for runtime
    /// - `time_limit`: optional time limit for function to return.
    ///   this can be extremely expensive so only use if absolutely required.
    pub fn try_new(
        path: Option<PathBuf>,
        inherit_stdout: bool,
        inherit_stderr: bool,
        memory_limit: Option<usize>,
        time_limit: Option<TimeLimit>,
    ) -> Result<Self> {
        let engine = Engine::new(Config::default().epoch_interruption(time_limit.is_some()))?;
        let module = match path {
            Some(path) => Module::from_file(&engine, path)?,
            None => Module::from_binary(&engine, include_bytes!("../../../quickjs.wasm"))?,
        };
        Ok(Self {
            engine,
            module,
            inherit_stdout,
            inherit_stderr,
            memory_limit: memory_limit.unwrap_or(usize::MAX),
            time_limit,
        })
    }
}

impl Debug for QuickJS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuickJS")
            .field("inherit_stdout", &self.inherit_stdout)
            .field("inherit_stderr", &self.inherit_stderr)
            .field("memory_limit", &self.memory_limit)
            .field("time_limit", &self.time_limit)
            .finish()
    }
}
struct State {
    pub wasi: WasiCtx,
    pub limits: StoreLimits,
}

impl QuickJS {
    pub fn try_execute(&self, script: &str, data: Option<&str>) -> Result<Option<String>> {
        let script = script.as_bytes().to_vec();
        let script_size = script.len() as i32;
        let data = data
            .map(|data| data.as_bytes().to_vec())
            .unwrap_or_default();
        let data_size = data.len() as i32;
        let output = Arc::new(Mutex::new(None));

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |state: &mut State| &mut state.wasi)?;

        let mut wasi_ctx_builder = WasiCtxBuilder::new();
        if self.inherit_stdout {
            wasi_ctx_builder = wasi_ctx_builder.inherit_stdout();
        };
        if self.inherit_stderr {
            wasi_ctx_builder = wasi_ctx_builder.inherit_stderr();
        };

        let wasi = wasi_ctx_builder.build();

        let mut store = Store::new(
            &self.engine,
            State {
                wasi,
                limits: StoreLimitsBuilder::new()
                    .memory_size(self.memory_limit)
                    .instances(1)
                    .build(),
            },
        );
        store.limiter(move |state| &mut state.limits);

        if let Some(time_limit) = self.time_limit.clone() {
            // calculate number of iterations to met timeout
            let mut timeout = u32::try_from(
                time_limit.time_limit.as_micros() / time_limit.evaluation_interval.as_micros(),
            )?;
            store.epoch_deadline_callback(move |_| {
                assert!(timeout > 0);
                timeout -= 1;
                if timeout == 0 {
                    bail!("exceeds time limit");
                }
                Ok(UpdateDeadline::Continue(1))
            });

            store.set_epoch_deadline(1);

            let engine = store.engine().clone();
            thread::spawn(move || loop {
                thread::sleep(time_limit.evaluation_interval);
                engine.increment_epoch();
            });
        }

        let memory_type = MemoryType::new(1, None);
        Memory::new(&mut store, memory_type)?;

        linker.func_wrap(
            "host",
            "get_script_size",
            move |_: Caller<'_, State>| -> Result<i32> { Ok(script_size) },
        )?;

        linker.func_wrap(
            "host",
            "get_script",
            move |mut caller: Caller<'_, State>, ptr: i32| -> Result<()> {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return Err(anyhow!("failed to find host memory")),
                };
                let offset = ptr as u32 as usize;
                Ok(memory.write(&mut caller, offset, &script)?)
            },
        )?;

        linker.func_wrap(
            "host",
            "get_data_size",
            move |_: Caller<'_, State>| -> Result<i32> { Ok(data_size) },
        )?;

        linker.func_wrap(
            "host",
            "get_data",
            move |mut caller: Caller<'_, State>, ptr: i32| -> Result<()> {
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
            move |mut caller: Caller<'_, State>, ptr: i32, capacity: i32| -> Result<()> {
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
        let quickjs = QuickJS::try_new(None, false, false, None, None).unwrap();

        let script = r#"
            'quickjs' + 'wasm'
        "#;

        let result = quickjs.try_execute(script, None).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));
    }

    #[test]
    fn try_execute_data() {
        let quickjs = QuickJS::try_new(None, false, false, None, None).unwrap();

        let script = r#"
            'quickjs' + data.input
        "#;

        let data = r#"{"input": "wasm"}"#;

        let result = quickjs.try_execute(script, Some(data)).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));
    }

    #[test]
    fn try_throw_error() {
        let quickjs = QuickJS::try_new(None, false, false, None, None).unwrap();

        let script = r#"
            throw new Error('error');
        "#;

        match quickjs.try_execute(script, None) {
            Err(err) if err.root_cause().to_string().contains("exit status 1") => {}
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn try_execute_memory_limit() {
        let quickjs = QuickJS::try_new(None, false, false, Some(1024), None).unwrap();

        let script = r#"
            'quickjs' + data.input
        "#;

        match quickjs.try_execute(script, None) {
            Err(err)
                if err
                    .root_cause()
                    .to_string()
                    .contains("exceeds memory limit") => {}
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn try_execute_time_limit() {
        let quickjs = QuickJS::try_new(
            None,
            false,
            false,
            None,
            Some(TimeLimit {
                time_limit: Duration::from_secs(2),
                evaluation_interval: Duration::from_millis(10),
            }),
        )
        .unwrap();

        let script = r#"
            function sleep(milliseconds) {
                const date = Date.now();
                let currentDate = null;
                do {
                currentDate = Date.now();
                } while (currentDate - date < milliseconds);
            }
            sleep(5000);
        "#;

        match quickjs.try_execute(script, None) {
            Err(err) if err.root_cause().to_string().contains("exceeds time limit") => {}
            other => panic!("{:?}", other),
        }
    }
}
