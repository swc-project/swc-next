//! ECMAScript and TypeScript AST node definitions.
//!
//! This module is a Rust port of the AST design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

mod js;
mod jsx;
mod ts;

pub use js::*;
pub use jsx::*;
pub use ts::*;

/// Source type of a JavaScript or TypeScript file.
///
/// Determines whether the file is parsed as an ES module, a CommonJS module,
/// or a classic script.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum SourceType {
    #[default]
    Module = 0,
    Script = 1,
    CommonJs = 2,
    /// Resolve to a module when module syntax is present, otherwise a script.
    Unambiguous = 3,
}

impl SourceType {
    /// Returns the conventional lowercase name of this source type.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Script => "script",
            Self::CommonJs => "commonjs",
            Self::Unambiguous => "unambiguous",
        }
    }

    /// Infers the source type from a file path.
    ///
    /// Explicit module extensions (`.mjs` and `.mts`) are modules, explicit
    /// CommonJS extensions (`.cjs` and `.cts`) are CommonJS modules, and all
    /// other paths use content-based detection.
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".mjs") || path.ends_with(".mts") {
            Self::Module
        } else if path.ends_with(".cjs") || path.ends_with(".cts") {
            Self::CommonJs
        } else {
            Self::Unambiguous
        }
    }
}

/// Language variant for JavaScript or TypeScript files. Determines which
/// syntax features are enabled during parsing.
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
    /// Infers the language variant from a file path.
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

    /// Returns whether this language variant enables TypeScript syntax.
    pub fn is_typescript(self) -> bool {
        matches!(self, Self::Ts | Self::Tsx | Self::Dts)
    }

    /// Returns whether this language variant enables JSX syntax.
    pub fn is_jsx(self) -> bool {
        matches!(self, Self::Jsx | Self::Tsx)
    }
}
