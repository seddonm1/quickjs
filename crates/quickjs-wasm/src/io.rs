use anyhow::Result;
use quickjs_wasm_rs::{json, Context, Value};

#[link(wasm_import_module = "host")]
extern "C" {
    fn get_input(ptr: i32);
    fn get_input_size() -> i32;
    fn get_data(ptr: i32);
    fn get_data_size() -> i32;
    fn set_output(ptr: i32, size: i32);
}

/// gets the input from the host as a string
pub fn get_input_string() -> Result<Option<String>> {
    let input_size = unsafe { get_input_size() } as usize;

    if input_size == 0 {
        Ok(None)
    } else {
        let mut buf: Vec<u8> = Vec::with_capacity(input_size);
        let ptr = buf.as_mut_ptr();
        unsafe { get_input(ptr as i32) };

        let input_buf = unsafe { Vec::from_raw_parts(ptr, input_size, input_size) };

        Ok(Some(String::from_utf8(input_buf.to_vec())?))
    }
}

/// gets the input from the host as a string
pub fn get_input_value(context: &Context) -> Result<Option<Value>> {
    let input_size = unsafe { get_data_size() } as usize;

    if input_size == 0 {
        Ok(None)
    } else {
        let mut buf: Vec<u8> = Vec::with_capacity(input_size);
        let ptr = buf.as_mut_ptr();
        unsafe { get_data(ptr as i32) };

        let input_buf = unsafe { Vec::from_raw_parts(ptr, input_size, input_size) };

        Ok(Some(json::transcode_input(context, &input_buf)?))
    }
}

/// sets the output value on the host
pub fn set_output_value(output: Option<Value>) -> Result<()> {
    match output {
        Some(output) if !output.is_undefined() => {
            let output = json::transcode_output(output)?;

            let size = output.len() as i32;
            let ptr = output.as_ptr();

            unsafe {
                set_output(ptr as i32, size);
            };
        }
        _ => {
            unsafe {
                set_output(0, 0);
            };
        }
    }

    Ok(())
}
