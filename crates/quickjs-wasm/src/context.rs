use anyhow::Result;
use quickjs_wasm_rs::{Context, Value};
use std::io::Write;

/// set quickjs globals
pub fn set_quickjs_globals(context: &Context) -> anyhow::Result<()> {
    let global = context.global_object()?;
    let console_log_callback = context.wrap_callback(console_log_to(std::io::stdout()))?;
    let console_error_callback = context.wrap_callback(console_log_to(std::io::stderr()))?;
    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;
    global.set_property("console", console_object)?;

    Ok(())
}

/// console_log_to is used to allow the javascript functions console.log and console.error to
/// log to the stdout and stderr respectively.
fn console_log_to<T>(mut stream: T) -> impl FnMut(&Context, &Value, &[Value]) -> Result<Value>
where
    T: Write + 'static,
{
    move |ctx: &Context, _this: &Value, args: &[Value]| {
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                write!(stream, " ")?;
            }

            stream.write_all(arg.as_str()?.as_bytes())?;
        }

        writeln!(stream)?;
        ctx.undefined_value()
    }
}
