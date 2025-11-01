use rquickjs::{Ctx, Function, Object, Result, Type, Value};

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    unsafe fn get_script(ptr: i32);
    unsafe fn get_script_size() -> i32;
    unsafe fn set_output(ptr: i32, size: i32, error: i32);
}

/// Converts a QuickJS Value to a user-facing Rust String.
/// NOTE: This function must be called from within a `ctx.with(...)` block.
///
// FIX: The function must accept the active context `Ctx<'js>`, not the owner `&Context`.
pub fn value_to_string<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<String> {
    // We can now use `ctx` directly, as it is the active context handle.
    match value.type_of() {
        Type::String => value.get::<rquickjs::String>()?.to_string(),
        Type::Bool | Type::Int | Type::Float | Type::BigInt | Type::Symbol | Type::Function | Type::Constructor => {
            let globals = ctx.globals();
            let string_fn: Function = globals.get("String")?;
            string_fn.call::<_, String>((value.clone(),))
        }
        Type::Null => Ok("null".to_string()),
        Type::Undefined => Ok("undefined".to_string()),
        Type::Object | Type::Array => {
            let globals = ctx.globals();
            let json: Object = globals.get("JSON")?;
            let stringify: Function = json.get("stringify")?;
            stringify.call::<_, String>((value.clone(),))
        }
        Type::Exception => {
            let exc = value.as_exception().unwrap();
            Ok(format!("{}", exc))
        }
        Type::Uninitialized => Ok("[uninitialized]".to_string()),
        Type::Module => Ok("[module]".to_string()),
        Type::Unknown => Ok("[unknown]".to_string()),
        Type::Promise => Ok("[promise]".to_string()),
    }
}

/// gets the script from the host as a string
pub fn get_input_script() -> Result<Option<String>> {
    let input_size = unsafe { get_script_size() } as usize;

    if input_size == 0 {
        Ok(None)
    } else {
        let mut buf: Vec<u8> = Vec::with_capacity(input_size);
        let ptr = buf.as_mut_ptr();
        unsafe { get_script(ptr as i32) };

        let input_buf = unsafe { Vec::from_raw_parts(ptr, input_size, input_size) };

        Ok(Some(String::from_utf8(input_buf.to_vec())?))
    }
}

/// sets the output value on the host
pub fn set_output_none() -> Result<()> {
    unsafe { set_output(0, 0, 0) }

    Ok(())
}

/// sets the output value on the host
pub fn set_output_value<'js>(ctx: &Ctx<'js>, output: Value<'js>) -> Result<()> {
    let output = value_to_string(ctx, &output).inspect_err(|err| println!("{err:?}"))?;
    let size = output.len() as i32;
    let ptr = output.as_ptr();

    unsafe { set_output(ptr as i32, size, 0) };

    Ok(())
}

/// sets the output value on the host as error
pub fn set_output_error(error: Value) -> Result<()> {
    let err = error.as_exception().unwrap().message().unwrap();
    let output = err.as_bytes();
    let size = output.len() as i32;
    let ptr = output.as_ptr();

    unsafe {
        set_output(ptr as i32, size, 1);
    }

    Ok(())
}
