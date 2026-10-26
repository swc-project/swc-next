use napi::{Env, JsStringUtf16, bindgen_prelude::ToNapiValue, sys};

pub(super) unsafe fn to_napi_value(
    env: sys::napi_env,
    value: String,
) -> napi::Result<sys::napi_value> {
    // JsStringUtf16::from_data rejects empty input.
    if value.is_empty() {
        return unsafe { String::to_napi_value(env, value) };
    }
    if value.len() > isize::MAX as usize / size_of::<u16>() {
        return Err(napi::Error::from_reason("UTF-16 string is too large"));
    }
    // One UTF-16 unit per UTF-8 byte is a safe upper bound, avoiding a sizing
    // pass. Preserve capacity so napi-rs can free the original allocation.
    let mut units = Vec::<u16>::with_capacity(value.len());
    // SAFETY: String is valid UTF-8 and the output has sufficient capacity.
    // N-API expects native-endian UTF-16 on every target.
    let written = unsafe {
        simdutf::convert_valid_utf8_to_utf16(value.as_ptr(), value.len(), units.as_mut_ptr())
    };
    unsafe { units.set_len(written) };
    let napi_env = Env::from(env);
    let result = JsStringUtf16::from_data(&napi_env, units)?;
    unsafe { JsStringUtf16::to_napi_value(env, result) }
}
