#![allow(static_mut_refs)]

mod io;

use anyhow::Result;
use rquickjs::{Context, Runtime, Value};
use std::mem::MaybeUninit;

static DEPENDENCIES: &str = include_str!("../dependencies/index.js");

struct RuntimeContext {
    // We don't need to store the Runtime separately if we use Context::full,
    // as the context will keep the runtime alive. However, it's good practice
    // to keep it explicitly to show ownership.
    #[allow(dead_code)]
    rt: Runtime,
    ctx: Context,
}

// 1. Use `static mut` with `MaybeUninit`.
// This allocates the memory for our RuntimeContext in the static data section of the binary.
// Wizer will save the state of this memory in its snapshot.
static mut RUNTIME: MaybeUninit<RuntimeContext> = MaybeUninit::uninit();

/// init() is executed by Wizer to create a snapshot after the QuickJS context has been initialized.
#[unsafe(export_name = "wizer.initialize")]
pub extern "C" fn init() {
    // Creating the runtime and context is safe.
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        // Pre-evaluate any expensive setup scripts.
        ctx.eval::<Value, _>(DEPENDENCIES).unwrap();
    });

    // 2. Write the initialized context into our static memory location.
    // This needs to be in an `unsafe` block because we are writing to a `static mut`.
    // The `write` method moves the `RuntimeContext` into the static variable,
    // ensuring it's not dropped at the end of this function.
    unsafe {
        RUNTIME.write(RuntimeContext { rt, ctx });
    }
}

fn main() -> Result<()> {
    match io::get_input_script()? {
        Some(input) => {
            // 3. Get a mutable reference to our Wizer-initialized runtime.
            // This is `unsafe` because the compiler can't prove that `init` was called.
            // With Wizer, we know it has been.
            let rt_ctx = unsafe { RUNTIME.assume_init_mut() };

            // 4. Use the existing context instead of creating a new one.
            rt_ctx.ctx.with(|ctx| {
                match ctx.eval::<Value, _>(input.as_bytes()) {
                    Ok(res) => io::set_output_value(&ctx, res)?,
                    Err(_) => io::set_output_error(ctx.catch())?,
                };
                Ok(())
            })
        }
        None => Ok(io::set_output_none()?),
    }
}
