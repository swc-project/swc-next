mod js;
mod jsx;
mod ts;

pub use js::*;
pub use jsx::*;
pub use ts::*;

use crate::{Ast, ExtraData, ExtraDataCompact};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum SourceType {
    Script,
    #[default]
    Module,
}

impl SourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Module => "module",
        }
    }

    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".cjs") || path.ends_with(".cts") {
            Self::Script
        } else {
            Self::Module
        }
    }
}

impl<'a> ExtraDataCompact<'a> for SourceType {
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute(data.other as u8) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum Lang {
    #[default]
    Js,
    Ts,
    Jsx,
    Tsx,
    Dts,
}

impl Lang {
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts") {
            Self::Dts
        } else if path.ends_with(".tsx") {
            Self::Tsx
        } else if path.ends_with(".ts") || path.ends_with(".mts") || path.ends_with(".cts") {
            Self::Ts
        } else if path.ends_with(".jsx") {
            Self::Jsx
        } else {
            Self::Js
        }
    }

    pub fn is_typescript(self) -> bool {
        matches!(self, Self::Ts | Self::Tsx | Self::Dts)
    }

    pub fn is_jsx(self) -> bool {
        matches!(self, Self::Jsx | Self::Tsx)
    }
}
