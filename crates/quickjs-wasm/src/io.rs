use anyhow::Result;
use quickjs_wasm_rs::{Deserializer, JSContextRef, JSValueRef, Serializer};

#[link(wasm_import_module = "host")]
extern "C" {
    fn get_script(ptr: i32);
    fn get_script_size() -> i32;
    fn get_data(ptr: i32);
    fn get_data_size() -> i32;
    fn set_output(ptr: i32, size: i32, error: i32);
}

/// Transcodes a byte slice containing a JSON encoded payload into a [`JSValueRef`].
///
/// Arguments:
/// * `context` - A reference to the [`JSContextRef`] that will contain the
///   returned [`JSValueRef`].
/// * `bytes` - A byte slice containing a JSON encoded payload.
pub fn transcode_input<'a>(context: &'a JSContextRef, bytes: &[u8]) -> Result<JSValueRef<'a>> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let mut serializer = Serializer::from_context(context)?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}

/// Transcodes a [`JSValueRef`] into a JSON encoded byte vector.
pub fn transcode_output(val: JSValueRef) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = serde_json::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
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

/// gets the data from the host as a JSValueRef
pub fn get_input_data(context: &JSContextRef) -> Result<Option<JSValueRef>> {
    let input_size = unsafe { get_data_size() } as usize;

    if input_size == 0 {
        Ok(None)
    } else {
        let mut buf: Vec<u8> = Vec::with_capacity(input_size);
        let ptr = buf.as_mut_ptr();
        unsafe { get_data(ptr as i32) };

        let input_buf = unsafe { Vec::from_raw_parts(ptr, input_size, input_size) };

        Ok(Some(transcode_input(context, &input_buf)?))
    }
}

/// sets the output value on the host
pub fn set_output_value(output: Result<Option<JSValueRef>>) -> Result<()> {
    match output {
        Ok(None) => unsafe {
            set_output(0, 0, 0);
        },
        Ok(Some(output)) => {
            let output = transcode_output(output)?;

            let size = output.len() as i32;
            let ptr = output.as_ptr();

            unsafe {
                set_output(ptr as i32, size, 0);
            }
        }
        Err(err) => {
            let err = err.to_string();

            let output = err.as_bytes();
            let size = output.len() as i32;
            let ptr = output.as_ptr();

            unsafe {
                set_output(ptr as i32, size, 1);
            };
        }
    }
    Ok(())
}
