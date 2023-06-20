#[cfg(feature = "console")]
mod context;
mod io;

use anyhow::Result;
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::JSContextRef;

static mut JS_CONTEXT: OnceCell<JSContextRef> = OnceCell::new();
static SCRIPT_NAME: &str = "script.js";

/// init() is executed by wizer to create a snapshot after the quickjs context has been initialized.
///
/// it also binds the console.log and console.error functions so they can be used for debugging in the
/// user script.
#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let context = JSContextRef::default();

        // add globals to the quickjs instance if enabled
        #[cfg(feature = "console")]
        context::set_quickjs_globals(&context).unwrap();

        JS_CONTEXT.set(context).unwrap();
    }
}

fn main() -> Result<()> {
    match io::get_input_script()? {
        Some(input) => {
            let context = unsafe { JS_CONTEXT.get_or_init(JSContextRef::default) };

            if let Some(value) = io::get_input_data(context)? {
                context.global_object()?.set_property("data", value)?;
            }

            let output = context.eval_global(SCRIPT_NAME, &input)?;
            io::set_output_value(Some(output))
        }
        None => io::set_output_value(None),
    }
}
