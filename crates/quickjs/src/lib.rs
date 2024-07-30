use anyhow::{anyhow, bail, Result};
use std::{
    fmt::Debug,
    path::PathBuf,
    sync::mpsc::sync_channel,
    thread::{self},
    time::Duration,
};
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::*;

static PAGE_SIZE: u32 = 65536;
static EPOCH_INTERVAL: u64 = 100;

/// A Rust wrapper around the QuickJS JavaScript engine.
///
/// This struct represents a running instance of the QuickJS engine, along with its module and configuration options.
pub struct QuickJS {
    /// The underlying QuickJS engine instance.
    engine: Engine,
    /// The module loaded into the engine.
    module: Module,
    /// Whether to inherit standard output from the parent process.
    inherit_stdout: bool,
    /// Whether to inherit standard error from the parent process.
    inherit_stderr: bool,
    /// Optional memory limit for the engine in bytes.
    memory_limit: Option<u32>,
    /// Optional time limit for the engine. If set, will be used to interrupt long-running scripts and prevent them from consuming excessive CPU time.
    time_limit: Option<TimeLimit>,
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

impl QuickJS {
    /// Creates a new instance of `QuickJS` with the specified options.
    ///
    /// # Arguments
    ///
    /// * `path`: The path to the JavaScript module file, or `None` for an embedded default module.
    /// * `inherit_stdout`: Whether to inherit standard output from the parent process.
    /// * `inherit_stderr`: Whether to inherit standard error from the parent process.
    /// * `memory_limit`: Optional memory limit for the engine in bytes.
    /// * `time_limit`: Optional time limit for the engine. If set, will be used to interrupt long-running scripts and prevent them from consuming excessive CPU time.
    ///
    /// # Returns
    ///
    /// A `Result` containing an instance of `QuickJS`, or an error if there was a problem creating it.
    pub fn try_new(
        path: Option<PathBuf>,
        inherit_stdout: bool,
        inherit_stderr: bool,
        memory_limit: Option<u32>,
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
            memory_limit,
            time_limit,
        })
    }
}

/// A builder for creating a `QuickJS` instance.
///
/// This struct allows you to configure various options and settings before building a `QuickJS` instance.
#[derive(Default)]
pub struct QuickJSBuilder {
    /// The path to a custom module file (optional).
    module: Option<PathBuf>,
    /// Whether to inherit standard output from the parent process (default: false).
    inherit_stdout: Option<bool>,
    /// Whether to inherit standard error from the parent process (default: false).
    inherit_stderr: Option<bool>,
    /// Optional memory limit for the engine in bytes.
    memory_limit: Option<u32>,
    /// Optional time limit for the engine. If set, will be used to interrupt long-running scripts and prevent them from consuming excessive CPU time.
    time_limit: Option<TimeLimit>,
}

impl QuickJSBuilder {
    /// Creates a new `QuickJSBuilder` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the path to a custom module file.
    ///
    /// If no module is provided, the engine will use its built-in module by default.
    pub fn with_module(mut self, path: PathBuf) -> Self {
        self.module = Some(path);
        self
    }

    /// Controls whether to inherit standard output from the parent process.
    pub fn with_inherit_stdout(mut self, inherit: bool) -> Self {
        self.inherit_stdout = Some(inherit);
        self
    }

    /// Controls whether to inherit standard error from the parent process.
    pub fn with_inherit_stderr(mut self, inherit: bool) -> Self {
        self.inherit_stderr = Some(inherit);
        self
    }

    /// Sets the memory limit for the engine in bytes.
    pub fn with_memory_limit(mut self, limit: u32) -> Self {
        self.memory_limit = Some(limit);
        self
    }

    /// Sets the time limit for the engine. If set, will be used to interrupt long-running scripts and prevent them from consuming excessive CPU time.
    pub fn with_time_limit(mut self, limit: TimeLimit) -> Self {
        self.time_limit = Some(limit);
        self
    }

    /// Builds a `QuickJS` instance from the current configuration settings.
    ///
    /// This method creates and returns a new `QuickJS` instance based on the settings provided through this builder.
    pub fn build(&self) -> Result<QuickJS> {
        QuickJS::try_new(
            self.module.clone(),
            self.inherit_stdout.unwrap_or(false),
            self.inherit_stderr.unwrap_or(false),
            self.memory_limit,
            self.time_limit.clone(),
        )
    }
}

#[derive(Clone, Debug)]
/// Time limit for QuickJS execution.
///
/// This struct represents a time limit for QuickJS execution. It allows setting
/// both a total execution time limit and an evaluation interval to check if the
/// execution is still within the allowed time frame.
pub struct TimeLimit {
    /// Total execution time limit in milliseconds.
    pub limit: Duration,
    /// Evaluation interval to check if the execution is still within the allowed time frame.
    pub evaluation_interval: Duration,
}

impl TimeLimit {
    /// Creates a new `TimeLimit` with the specified total execution time limit.
    ///
    /// # Arguments
    ///
    /// * `limit`: Total execution time limit in milliseconds.
    pub fn new(limit: Duration) -> Self {
        Self {
            limit,
            evaluation_interval: Duration::from_micros(EPOCH_INTERVAL),
        }
    }

    /// Creates a new `TimeLimit` with the specified total execution time limit and evaluation interval.
    ///
    /// # Arguments
    ///
    /// * `limit`: Total execution time limit in milliseconds.
    /// * `evaluation_interval`: Evaluation interval to check if the execution is still within the allowed time frame.
    pub fn with_evaluation_interval(mut self, evaluation_interval: Duration) -> Self {
        self.evaluation_interval = evaluation_interval;
        self
    }
}

struct State {
    pub wasi: WasiCtx,
    pub limits: StoreLimits,
}

impl QuickJS {
    /// Attempts to execute the given JavaScript code with optional input data.
    ///
    /// This method sets up a WASI context and executes the provided JavaScript code in that context. If `data` is provided, it will be passed to the script as standard input.
    ///
    /// # Arguments
    ///
    /// * `script`: The JavaScript code to execute as a string.
    /// * `data`: Optional input data to pass to the script as standard input.
    ///
    /// # Returns
    ///
    /// If execution is successful, it returns `Some(String)` with the output  or None if no output is returned from the JavaScript context.
    pub fn try_execute(&self, script: &str, data: Option<&str>) -> Result<Option<String>> {
        // Convert the script string to a byte vector for later use
        let script = script.as_bytes().to_vec();

        // Get the size of the script as an i32 (for WASI API calls)
        let script_size = script.len() as i32;

        // Optionally convert the data string to a byte vector and set its default value if it's not provided
        let data = data
            .map(|data| data.as_bytes().to_vec())
            .unwrap_or_default();

        // Get the size of the data as an i32 (for WASI API calls)
        let data_size = data.len() as i32;

        // Create a new linker for the engine
        let mut linker = Linker::new(&self.engine);

        // Add the WASI library to the linker
        wasi_common::sync::add_to_linker(&mut linker, |state: &mut State| &mut state.wasi)?;

        // Build a new WASI context builder
        let mut wasi_ctx_builder = WasiCtxBuilder::new();

        // Inherit stdout if requested by the user
        if self.inherit_stdout {
            wasi_ctx_builder.inherit_stdout();
        };

        // Inherit stderr if requested by the user
        if self.inherit_stderr {
            wasi_ctx_builder.inherit_stderr();
        };

        // Build the WASI context with the provided options
        let wasi = wasi_ctx_builder.build();

        // Determine memory type and limits based on self.memory_limit.
        let (memory_type, limits) = match self.memory_limit {
            // If self.memory_limit is Some, calculate memory type and limits based on PAGE_SIZE.
            Some(memory_limit) => (
                MemoryType::new(memory_limit / PAGE_SIZE, Some(memory_limit / PAGE_SIZE)),
                StoreLimitsBuilder::new()
                    .instances(1)
                    .memory_size(memory_limit as usize)
                    .build(),
            ),
            // If self.memory_limit is None, use default values for memory type and limits.
            None => (
                MemoryType::new(1, None),
                StoreLimitsBuilder::new().instances(1).build(),
            ),
        };

        // Create a new store instance with the engine and initial state.
        let mut store = Store::new(&self.engine, State { wasi, limits });

        // Set the limiter for the store to access its limits.
        store.limiter(move |state| &mut state.limits);

        // If self.time_limit is Some, set up a thread to increment the epoch at regular intervals.
        if let Some(time_limit) = &self.time_limit {
            // Calculate evaluation interval from time limit.
            let evaluation_interval = time_limit.evaluation_interval;
            // Clone engine instance for use in separate thread.
            let engine_clone = self.engine.clone();
            // Start new thread to increment epoch every evaluation interval.
            thread::spawn(move || loop {
                thread::sleep(evaluation_interval);
                engine_clone.increment_epoch();
            });

            // Calculate initial epoch limit from time limit.
            let mut epoch_limit = u32::try_from(
                time_limit.limit.as_micros() / time_limit.evaluation_interval.as_micros(),
            )?;

            // Set up callback for when the epoch deadline is reached.
            store.epoch_deadline_callback(move |_| {
                // If epoch limit reaches 0, return error.
                if epoch_limit == 0 {
                    bail!("exceeds time limit");
                }
                // Decrement epoch limit and continue evaluation.
                epoch_limit -= 1;
                Ok(UpdateDeadline::Continue(1))
            });

            // Set initial epoch deadline to 1.
            store.set_epoch_deadline(1);
        }

        // Create new memory instance with the store and calculated memory type.
        Memory::new(&mut store, memory_type)?;

        // Wraps the host function to retrieve the size of the script.
        // This function is exposed as `get_script_size` in the JavaScript context.
        linker.func_wrap(
            "host",
            "get_script_size",
            move |_: Caller<'_, State>| -> Result<i32> { Ok(script_size) },
        )?;

        // Wraps the host function to retrieve the script data.
        // This function is exposed as `get_script` in the JavaScript context.
        linker.func_wrap(
            "host",
            "get_script",
            move |mut caller: Caller<'_, State>, ptr: i32| -> Result<()> {
                // The memory export from the host environment.
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return Err(anyhow!("failed to find host memory")),
                };

                // The offset in bytes at which to write the script data.
                let offset = ptr as u32 as usize;

                Ok(memory.write(&mut caller, offset, &script)?)
            },
        )?;

        // Wraps the host function to retrieve the size of the input data.
        // This function is exposed as `get_data_size` in the JavaScript context.
        linker.func_wrap(
            "host",
            "get_data_size",
            move |_: Caller<'_, State>| -> Result<i32> { Ok(data_size) },
        )?;

        // Wraps the host function to retrieve the input data.
        // This function is exposed as `get_data` in the JavaScript context.
        linker.func_wrap(
            "host",
            "get_data",
            move |mut caller: Caller<'_, State>, ptr: i32| -> Result<()> {
                // The memory export from the host environment.
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return Err(anyhow!("failed to find host memory")),
                };

                // The offset in bytes at which to write the input data.
                let offset = ptr as u32 as usize;

                Ok(memory.write(&mut caller, offset, &data)?)
            },
        )?;

        // A simulated one-shot channel to wait for the script to complete and retrieve the result.
        let (sender, receiver) = sync_channel(1);

        // Wraps the host function to retrieve the output data from the host memory.
        // This function is exposed as `set_output` in the JavaScript context.
        linker.func_wrap(
            "host",
            "set_output",
            move |mut caller: Caller<'_, State>,
                  ptr: i32,
                  capacity: i32,
                  error: i32|
                  -> Result<()> {
                // Check for invalid capacity
                if capacity == 0 {
                    // If the capacity is zero, send None to the guest.
                    sender.send(None).unwrap();
                } else {
                    // Get the host memory object from the caller's exports.
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(memory)) => Ok(memory),
                        _ => Err(anyhow!("failed to find host memory")),
                    }?;

                    // Calculate the offset of the string in host memory.
                    let offset = ptr as u32 as usize;

                    // Allocate a buffer to store the read string.
                    let mut buffer: Vec<u8> = vec![0; capacity as usize];

                    // Read the string from host memory into the buffer.
                    memory.read(&caller, offset, &mut buffer)?;

                    // Convert the buffer to a string and try to parse it as UTF-8.
                    let result = String::from_utf8(buffer)?;

                    // If an error occurred while reading the string, send the error back to the guest; otherwise, send the read string back.
                    if error == 0 {
                        sender.send(Some(Ok(result))).unwrap();
                    } else {
                        sender.send(Some(Err(anyhow!(result)))).unwrap();
                    };
                };

                Ok(())
            },
        )?;

        // Create a new module in the store with an empty name and link it to our current module.
        linker.module(&mut store, "", &self.module)?;

        // Call the module's default entrypoint.
        linker
            .get_default(&mut store, "")?
            .typed::<(), ()>(&store)?
            .call(&mut store, ())?;

        // Receive any message that was sent to this module and return it (if anything was sent)
        receiver.recv()?.transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_execute() -> Result<()> {
        let quickjs = QuickJSBuilder::new().build()?;

        let script = r#"
            'quickjs' + 'wasm'
        "#;

        let result = quickjs.try_execute(script, None).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));

        Ok(())
    }

    #[test]
    fn try_execute_data() -> Result<()> {
        let quickjs = QuickJSBuilder::new().build()?;

        let script = r#"
            'quickjs' + data.input
        "#;

        let data = r#"{"input": "wasm"}"#;

        let result = quickjs.try_execute(script, Some(data)).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));

        Ok(())
    }

    #[test]
    fn try_throw_error() -> Result<()> {
        let quickjs = QuickJSBuilder::new().build()?;

        let script = r#"
            throw new Error('myerror');
        "#;

        match quickjs.try_execute(script, None) {
            Err(err) if err.to_string().contains("Uncaught Error: myerror") => {}
            other => panic!("{:?}", other),
        }

        Ok(())
    }

    #[test]
    fn try_execute_memory_limit_normal() -> Result<()> {
        let quickjs = QuickJSBuilder::new().with_memory_limit(4194304).build()?;

        let script = r#"
            'quickjs' + 'wasm'
        "#;

        let result = quickjs.try_execute(script, None).unwrap();

        assert_eq!(result, Some("\"quickjswasm\"".to_string()));

        Ok(())
    }

    #[test]
    fn try_execute_memory_limit_exceed() -> Result<()> {
        let quickjs = QuickJSBuilder::new().with_memory_limit(4194304).build()?;

        let script = r#"
            let memory = [];
            while (true) {
                memory.push("allocate");
            }
        "#;

        match quickjs.try_execute(script, None) {
            Err(err) if err.to_string().contains("out of memory") => {}
            other => panic!("{:?}", other),
        }

        Ok(())
    }

    #[test]
    fn try_execute_time_limit() -> Result<()> {
        let quickjs = QuickJSBuilder::new()
            .with_time_limit(
                TimeLimit::new(Duration::from_secs(2))
                    .with_evaluation_interval(Duration::from_millis(100)),
            )
            .build()?;

        let script = r#"
            function sleep(milliseconds) {
                const startDate = Date.now();
                let currentDate = Date.now();
                do {
                    currentDate = Date.now();
                } while (currentDate - startDate < milliseconds);
            }
            sleep(30000);
        "#;

        match quickjs.try_execute(script, None) {
            Err(err) if err.root_cause().to_string().contains("exceeds time limit") => {}
            other => panic!("{:?}", other),
        }

        Ok(())
    }
}
