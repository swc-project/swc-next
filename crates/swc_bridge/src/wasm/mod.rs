mod runtime;

use std::str;

pub use runtime::{alloc, free};

/// Convert a raw WebAssembly ABI value into a bridge wire value.
///
/// # Safety
///
/// `Self::Abi` must satisfy the memory and scalar representation required by
/// the implementation.
pub unsafe trait FromWasmAbi: Sized {
    type Abi;

    /// # Safety
    ///
    /// `value` must satisfy this implementation's ABI contract.
    unsafe fn from_wasm_abi(value: Self::Abi) -> Option<Self>;
}

/// Convert a bridge wire value into a raw WebAssembly ABI value.
pub trait IntoWasmAbi {
    type Abi;

    fn into_wasm_abi(self) -> Self::Abi;
}

/// Encode a successful value for a fallible WebAssembly bridge call.
pub trait IntoWasmResult {
    fn into_wasm_result(self) -> u32;
}

macro_rules! scalar_wasm_abi {
    ($($ty:ty),* $(,)?) => {
        $(
            unsafe impl FromWasmAbi for $ty {
                type Abi = Self;

                unsafe fn from_wasm_abi(value: Self::Abi) -> Option<Self> {
                    Some(value)
                }
            }

            impl IntoWasmAbi for $ty {
                type Abi = Self;

                fn into_wasm_abi(self) -> Self::Abi {
                    self
                }
            }
        )*
    };
}

scalar_wasm_abi!((), u32, i32, f64);

unsafe impl FromWasmAbi for bool {
    type Abi = u32;

    unsafe fn from_wasm_abi(value: Self::Abi) -> Option<Self> {
        match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

impl IntoWasmAbi for bool {
    type Abi = u32;

    fn into_wasm_abi(self) -> Self::Abi {
        u32::from(self)
    }
}

unsafe impl FromWasmAbi for String {
    type Abi = u32;

    unsafe fn from_wasm_abi(pointer: Self::Abi) -> Option<Self> {
        let bytes = unsafe { runtime::borrow_payload(pointer)? };
        str::from_utf8(bytes).ok().map(str::to_owned)
    }
}

impl IntoWasmAbi for String {
    type Abi = u32;

    fn into_wasm_abi(self) -> Self::Abi {
        runtime::allocate_payload(self.as_bytes())
    }
}

unsafe impl FromWasmAbi for Vec<u8> {
    type Abi = u32;

    unsafe fn from_wasm_abi(pointer: Self::Abi) -> Option<Self> {
        Some(unsafe { runtime::borrow_payload(pointer)? }.to_vec())
    }
}

impl IntoWasmAbi for Vec<u8> {
    type Abi = u32;

    fn into_wasm_abi(self) -> Self::Abi {
        runtime::allocate_payload(&self)
    }
}

macro_rules! scalar_wasm_result {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoWasmResult for $ty {
                fn into_wasm_result(self) -> u32 {
                    runtime::allocate_tagged(0, &self.to_le_bytes())
                }
            }
        )*
    };
}

scalar_wasm_result!(u32, i32, f64);

impl IntoWasmResult for () {
    fn into_wasm_result(self) -> u32 {
        runtime::allocate_tagged(0, &[])
    }
}

impl IntoWasmResult for bool {
    fn into_wasm_result(self) -> u32 {
        runtime::allocate_tagged(0, &u32::from(self).to_le_bytes())
    }
}

impl IntoWasmResult for String {
    fn into_wasm_result(self) -> u32 {
        runtime::allocate_tagged(0, self.as_bytes())
    }
}

impl IntoWasmResult for Vec<u8> {
    fn into_wasm_result(self) -> u32 {
        runtime::allocate_tagged(0, &self)
    }
}

#[cold]
#[inline(never)]
pub fn invalid_abi() -> ! {
    core::arch::wasm32::unreachable()
}

pub fn error_result(message: &str) -> u32 {
    runtime::allocate_tagged(1, message.as_bytes())
}
