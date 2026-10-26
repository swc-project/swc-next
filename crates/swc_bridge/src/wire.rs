#[cfg(not(target_family = "wasm"))]
use napi::bindgen_prelude::{Buffer, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};
#[cfg(not(target_family = "wasm"))]
use napi::{Env, sys};

#[doc(hidden)]
pub mod private {
    pub trait Sealed {}
}

/// A value supported directly by the bridge transport.
pub trait BridgeWire: private::Sealed {}

/// Decode a bridge wire value into a semantic API input.
pub trait BridgeDecode: Sized {
    type Wire: BridgeWire;

    fn decode(value: Self::Wire) -> Option<Self>;
}

/// Encode a semantic API output as a bridge wire value.
pub trait BridgeEncode {
    type Wire: BridgeWire;

    fn encode(self) -> Self::Wire;
}

macro_rules! identity_wire {
    ($($ty:ty),* $(,)?) => {
        $(
            impl private::Sealed for $ty {}
            impl BridgeWire for $ty {}

            impl BridgeDecode for $ty {
                type Wire = Self;

                fn decode(value: Self::Wire) -> Option<Self> {
                    Some(value)
                }
            }

            impl BridgeEncode for $ty {
                type Wire = Self;

                fn encode(self) -> Self::Wire {
                    self
                }
            }
        )*
    };
}

identity_wire!((), u32, i32, f64, bool, String, Vec<u8>);

/// Map a logical bridge wire value to the type exposed through N-API.
#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub trait NativeAbi: BridgeWire {
    type Abi: FromNapiValue + ToNapiValue + TypeName + ValidateNapiValue;

    fn from_native(value: Self::Abi) -> Self;
    fn into_native(self) -> Self::Abi;
}

#[cfg(not(target_family = "wasm"))]
macro_rules! identity_native_abi {
    ($($ty:ty),* $(,)?) => {
        $(
            impl NativeAbi for $ty {
                type Abi = Self;

                fn from_native(value: Self::Abi) -> Self {
                    value
                }

                fn into_native(self) -> Self::Abi {
                    self
                }
            }
        )*
    };
}

#[cfg(not(target_family = "wasm"))]
identity_native_abi!((), u32, i32, f64, bool);

/// An owned string with a bounded short-input conversion at the N-API boundary.
#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub struct NativeString(String);

#[cfg(not(target_family = "wasm"))]
impl TypeName for NativeString {
    fn type_name() -> &'static str {
        <String as TypeName>::type_name()
    }

    fn value_type() -> napi::ValueType {
        <String as TypeName>::value_type()
    }
}

#[cfg(not(target_family = "wasm"))]
impl ValidateNapiValue for NativeString {}

#[cfg(not(target_family = "wasm"))]
impl FromNapiValue for NativeString {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        value: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let mut units = 0;
        let status = unsafe {
            napi::sys::napi_get_value_string_utf16(env, value, std::ptr::null_mut(), 0, &mut units)
        };
        if status != napi::sys::Status::napi_ok {
            // Keep napi-rs' detailed type errors and exception handling.
            return unsafe { String::from_napi_value(env, value) }.map(Self);
        }
        // Bound excess capacity for short strings; long inputs retain napi-rs'
        // exact-size allocation. UTF-8 requires at most three bytes per UTF-16
        // code unit, including replacement characters for unpaired surrogates.
        const MAX_BUFFER_SIZE: usize = 4096;
        if units > (MAX_BUFFER_SIZE - 1) / 3 {
            return unsafe { String::from_napi_value(env, value) }.map(Self);
        }
        let capacity = units * 3 + 1;
        let mut bytes = Vec::<u8>::with_capacity(capacity);
        let mut written = 0;
        let status = unsafe {
            napi::sys::napi_get_value_string_utf8(
                env,
                value,
                bytes.as_mut_ptr().cast(),
                capacity,
                &mut written,
            )
        };
        if status != napi::sys::Status::napi_ok {
            return unsafe { String::from_napi_value(env, value) }.map(Self);
        }
        // N-API initialized exactly `written` UTF-8 bytes, excluding the
        // terminator. The bound above leaves room for the complete string.
        unsafe { bytes.set_len(written) };
        Ok(Self(unsafe { String::from_utf8_unchecked(bytes) }))
    }
}

#[cfg(not(target_family = "wasm"))]
impl ToNapiValue for NativeString {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        value: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        let value = value.0;
        if value.is_empty() {
            return unsafe { String::to_napi_value(env, value) };
        }
        let utf16_len = simdutf::utf16_length_from_utf8(value.as_bytes());
        let napi_env = Env::from(env);
        let mut result = std::ptr::null_mut();
        // For valid UTF-8, only ASCII has the same byte and UTF-16 unit counts.
        if utf16_len == value.len() {
            let bytes = value.into_bytes();
            napi_env.adjust_external_memory(bytes.capacity() as i64)?;
            let (data, len, capacity) = bytes.into_raw_parts();
            // The hint stores only a capacity, avoiding a separate heap allocation.
            let hint = capacity as *mut std::ffi::c_void;
            // SAFETY: Ownership transfers to the finalizer on success, including
            // when N-API copies and invokes it synchronously. No later buffer access.
            let status = unsafe {
                sys::node_api_create_external_string_latin1(
                    env,
                    data.cast(),
                    len as isize,
                    Some(finalize_string_buffer::<u8>),
                    hint,
                    &mut result,
                    std::ptr::null_mut(),
                )
            };
            if status != sys::Status::napi_ok {
                // N-API does not invoke the finalizer on failure; we still own it.
                unsafe { finalize_string_buffer::<u8>(env.cast(), data.cast(), hint) };
            }
            napi::check_status!(status, "Failed to create external Latin-1 string")?;
            return Ok(result);
        }
        if utf16_len > isize::MAX as usize / size_of::<u16>() {
            return Err(napi::Error::from_reason("UTF-16 string is too large"));
        }
        let mut units = Vec::<u16>::with_capacity(utf16_len);
        // SAFETY: String is valid UTF-8 and the output has sufficient capacity.
        // N-API expects native-endian UTF-16 on every target.
        let written = unsafe {
            simdutf::convert_valid_utf8_to_utf16(value.as_ptr(), value.len(), units.as_mut_ptr())
        };
        debug_assert_eq!(written, utf16_len);
        unsafe { units.set_len(written) };
        drop(value);
        napi_env.adjust_external_memory((units.capacity() * size_of::<u16>()) as i64)?;
        let (data, len, capacity) = units.into_raw_parts();
        let hint = capacity as *mut std::ffi::c_void;
        // SAFETY: The allocation contains native-endian UTF-16. The finalizer
        // owns it on success and may free it synchronously if N-API copies it.
        let status = unsafe {
            sys::node_api_create_external_string_utf16(
                env,
                data,
                len as isize,
                Some(finalize_string_buffer::<u16>),
                hint,
                &mut result,
                std::ptr::null_mut(),
            )
        };
        if status != sys::Status::napi_ok {
            unsafe { finalize_string_buffer::<u16>(env.cast(), data.cast(), hint) };
        }
        napi::check_status!(status, "Failed to create external UTF-16 string")?;
        Ok(result)
    }
}

#[cfg(not(target_family = "wasm"))]
unsafe extern "C" fn finalize_string_buffer<T>(
    env: sys::node_api_basic_env,
    data: *mut std::ffi::c_void,
    hint: *mut std::ffi::c_void,
) {
    let capacity = hint as usize;
    let allocated_bytes = (capacity * size_of::<T>()) as i64;
    // SAFETY: data and capacity came from Vec::into_raw_parts. The only element
    // types are u8 and u16, so no element destructors require the original length.
    drop(unsafe { Vec::from_raw_parts(data.cast::<T>(), 0, capacity) });
    // External strings can be finalized after their environment has shut down.
    if !env.is_null() {
        let mut adjusted = 0;
        let _ = unsafe {
            sys::napi_adjust_external_memory(env.cast(), -allocated_bytes, &mut adjusted)
        };
    }
}

#[cfg(not(target_family = "wasm"))]
impl NativeAbi for String {
    type Abi = NativeString;

    fn from_native(value: Self::Abi) -> Self {
        value.0
    }

    fn into_native(self) -> Self::Abi {
        NativeString(self)
    }
}

#[cfg(not(target_family = "wasm"))]
impl NativeAbi for Vec<u8> {
    type Abi = Buffer;

    fn from_native(value: Self::Abi) -> Self {
        value.to_vec()
    }

    fn into_native(self) -> Self::Abi {
        self.into()
    }
}
