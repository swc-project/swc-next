use clap::{Parser, ValueEnum};

#[derive(Parser)]
pub struct Args {
    /// Embedded JavaScript or TypeScript fixture.
    #[arg(value_enum)]
    pub file: Input,

    #[arg(value_enum)]
    pub operation: Operation,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Input {
    #[value(name = "typescript.js")]
    TypeScript,
    #[value(name = "effect-schema-ast.ts")]
    EffectSchemaAst,
    #[value(name = "hono-types.ts")]
    HonoTypes,
}

impl Input {
    pub fn name(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript.js",
            Self::EffectSchemaAst => "effect-schema-ast.ts",
            Self::HonoTypes => "hono-types.ts",
        }
    }

    pub fn source(self) -> &'static str {
        match self {
            Self::TypeScript => include_str!("../files/typescript.js"),
            Self::EffectSchemaAst => include_str!("../files/effect-schema-ast.ts"),
            Self::HonoTypes => include_str!("../files/hono-types.ts"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Operation {
    Parse,
    IsolatedDeclaration,
    TypeStrip,
}
