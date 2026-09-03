#[cfg(not(target_family = "wasm"))]
use napi::bindgen_prelude::{Buffer, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};

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
identity_native_abi!((), u32, i32, f64, bool, String);

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
