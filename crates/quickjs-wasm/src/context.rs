use anyhow::Result;
use quickjs_wasm_rs::{CallbackArg, JSContextRef, JSValue};
use std::io::Write;

/// set quickjs globals
pub fn set_quickjs_globals(context: &JSContextRef) -> anyhow::Result<()> {
    let console_log_callback = context.wrap_callback(console_log_to(std::io::stdout()))?;
    let console_error_callback = context.wrap_callback(console_log_to(std::io::stderr()))?;

    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;

    let global = context.global_object()?;
    global.set_property("console", console_object)?;

    Ok(())
}

/// console_log_to is used to allow the javascript functions console.log and console.error to
/// log to the stdout and stderr respectively.
fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> Result<JSValue>
where
    T: Write + 'static,
{
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        // Write full string to in-memory destination before writing to stream since each write call to the stream
        // will invoke a hostcall.
        let mut log_line = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                log_line.push(' ');
            }
            let line = arg.to_string();
            log_line.push_str(&line);
        }

        writeln!(stream, "{log_line}")?;

        Ok(JSValue::Undefined)
    }
}
