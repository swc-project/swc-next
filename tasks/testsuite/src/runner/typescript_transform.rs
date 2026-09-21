use std::{
    any::Any,
    fmt::{Display, Formatter},
    panic::{AssertUnwindSafe, catch_unwind},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Decorator, Lang, Severity, SourceType, TsExportAssignment, Visit, VisitWith,
};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{
    normalize::Normalizer,
    typescript::{self, TypeScriptOptions},
};

use crate::cases::typescript_transform::{TypeScriptTransformMode, TypeScriptTransformUnit};

pub struct TypeScriptTransformRunner;

pub enum TypeScriptTransformCheckResult {
    Transformed(String),
    InputParseSkipped,
    InputSemanticSkipped,
    OtherTransformSkipped(OtherTransformReason),
    Failed(String),
    Panic(String),
}

#[derive(Clone, Copy)]
pub enum OtherTransformReason {
    Decorators,
    Module,
}

impl TypeScriptTransformRunner {
    pub const NAME: &'static str = "typescript_transform";

    pub fn check(
        mode: TypeScriptTransformMode,
        unit: &TypeScriptTransformUnit,
    ) -> TypeScriptTransformCheckResult {
        let result = catch_unwind(AssertUnwindSafe(|| transform_unit(mode, unit)));
        match result {
            Ok(Ok(output)) => TypeScriptTransformCheckResult::Transformed(output),
            Ok(Err(TransformUnitError::InputParse(_))) => {
                TypeScriptTransformCheckResult::InputParseSkipped
            }
            Ok(Err(TransformUnitError::InputSemantic(_))) => {
                TypeScriptTransformCheckResult::InputSemanticSkipped
            }
            Ok(Err(TransformUnitError::OtherTransform(reason))) => {
                TypeScriptTransformCheckResult::OtherTransformSkipped(reason)
            }
            Ok(Err(TransformUnitError::Failed(error))) => {
                TypeScriptTransformCheckResult::Failed(error)
            }
            Err(payload) => TypeScriptTransformCheckResult::Panic(panic_message(payload)),
        }
    }

    pub fn transform(
        mode: TypeScriptTransformMode,
        unit: &TypeScriptTransformUnit,
    ) -> Result<String, String> {
        let result = catch_unwind(AssertUnwindSafe(|| transform_unit(mode, unit)));
        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(error)) => Err(error.to_string()),
            Err(payload) => Err(format!("transform panicked: {}", panic_message(payload))),
        }
    }
}

fn transform_unit(
    mode: TypeScriptTransformMode,
    unit: &TypeScriptTransformUnit,
) -> Result<String, TransformUnitError> {
    let allocator = Allocator::new();
    let mut ast = parse_program(&allocator, &unit.code, unit.source_type, unit.lang, "input")
        .map_err(TransformUnitError::InputParse)?;
    let semantic_ret = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: true,
            build_module_record: false,
        },
    );
    if let Some(diagnostic) = semantic_ret
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(TransformUnitError::InputSemantic(format!(
            "semantic: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        )));
    }
    let program = ast.root_program();
    Normalizer::normalize(program, &mut ast);
    typescript::transform(
        program,
        &mut ast,
        &semantic_ret.semantic,
        TypeScriptOptions {
            verbatim_module_syntax: mode.verbatim_module_syntax(),
            preserve_const_enums: mode.preserve_const_enums(),
            ..Default::default()
        },
    );
    let mut detector = OtherTransformDetector::new(&ast);
    program.visit_with(&mut detector);
    if let Some(reason) = detector.reason() {
        return Err(TransformUnitError::OtherTransform(reason));
    }

    let output = print(&allocator, &ast, CodegenOptions::default())
        .code
        .to_owned();
    if unit.lang == Lang::Dts {
        if output.is_empty() {
            return Ok(output);
        }
        return Err(TransformUnitError::Failed(
            "declaration input emitted JavaScript".to_string(),
        ));
    }

    let output_lang = if unit.lang == Lang::Tsx {
        Lang::Jsx
    } else {
        Lang::Js
    };
    let output_allocator = Allocator::new();
    // Validate output with the same configured mode used for the input. An
    // unambiguous parse may resolve its AST to a concrete source type without
    // changing the caller's parsing contract.
    parse_program(
        &output_allocator,
        &output,
        unit.source_type,
        output_lang,
        "output",
    )
    .map_err(TransformUnitError::Failed)?;
    Ok(output)
}

enum TransformUnitError {
    InputParse(String),
    InputSemantic(String),
    OtherTransform(OtherTransformReason),
    Failed(String),
}

impl Display for TransformUnitError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputParse(error) | Self::InputSemantic(error) | Self::Failed(error) => {
                formatter.write_str(error)
            }
            Self::OtherTransform(reason) => write!(formatter, "requires {reason}"),
        }
    }
}

struct OtherTransformDetector<'a> {
    ast: &'a Ast<'a>,
    has_decorators: bool,
    has_export_assignment: bool,
}

impl<'a> OtherTransformDetector<'a> {
    fn new(ast: &'a Ast<'a>) -> Self {
        Self {
            ast,
            has_decorators: false,
            has_export_assignment: false,
        }
    }

    fn reason(&self) -> Option<OtherTransformReason> {
        if self.has_decorators {
            Some(OtherTransformReason::Decorators)
        } else if self.has_export_assignment {
            Some(OtherTransformReason::Module)
        } else {
            None
        }
    }
}

impl<'a> Visit<'a> for OtherTransformDetector<'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_decorator(&mut self, _node: Decorator) {
        self.has_decorators = true;
    }

    fn visit_ts_export_assignment(&mut self, _node: TsExportAssignment) {
        self.has_export_assignment = true;
    }
}

impl Display for OtherTransformReason {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decorators => formatter.write_str("the decorators transform"),
            Self::Module => formatter.write_str("a module transform"),
        }
    }
}

fn parse_program<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    lang: Lang,
    stage: &str,
) -> Result<Ast<'a>, String> {
    let result = Parser::init(
        allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if let Some(diagnostic) = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "{stage} parser: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }
    Ok(result.ast)
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}
