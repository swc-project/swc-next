use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_transformer::{TransformerOptions, typescript::TypeScriptOptions};

use super::{lang_from_path, read_test_content, typescript_case_paths};
use crate::cases::{Case, TransformCase, fixtures};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TypeScriptTransformMode {
    verbatim_module_syntax: bool,
    preserve_const_enums: bool,
}

impl TypeScriptTransformMode {
    pub const DEFAULT: Self = Self::new(false, false);
    pub const PRESERVE_CONST_ENUMS: Self = Self::new(false, true);
    pub const VERBATIM: Self = Self::new(true, false);
    pub const VERBATIM_PRESERVE_CONST_ENUMS: Self = Self::new(true, true);

    const fn new(verbatim_module_syntax: bool, preserve_const_enums: bool) -> Self {
        Self {
            verbatim_module_syntax,
            preserve_const_enums,
        }
    }

    pub fn name(self) -> &'static str {
        match (self.verbatim_module_syntax, self.preserve_const_enums) {
            (false, false) => "verbatimModuleSyntax: false, preserveConstEnums: false",
            (false, true) => "verbatimModuleSyntax: false, preserveConstEnums: true",
            (true, false) => "verbatimModuleSyntax: true, preserveConstEnums: false",
            (true, true) => "verbatimModuleSyntax: true, preserveConstEnums: true",
        }
    }

    pub fn verbatim_module_syntax(self) -> bool {
        self.verbatim_module_syntax
    }

    pub fn preserve_const_enums(self) -> bool {
        self.preserve_const_enums
    }
}

/// One TypeScript virtual file and one transform option combination.
pub struct TypeScriptTransformCase {
    path: PathBuf,
    relative_path: PathBuf,
    name: String,
    virtual_file_name: Option<String>,
    code: Arc<str>,
    mode: TypeScriptTransformMode,
    source_type: SourceType,
    lang: Lang,
}

impl TypeScriptTransformCase {
    pub fn read_all(mode_override: Option<TypeScriptTransformMode>) -> Result<Vec<Self>, String> {
        let test_dir = fixtures().join("typescript/tsc/testdata/tests/cases");
        let mut cases = Vec::new();
        for path in typescript_case_paths(&test_dir)? {
            let parsed = read_test_content(&path)?;
            let modes = transform_modes(
                parsed.global_option("verbatimmodulesyntax"),
                parsed.global_option("preserveconstenums"),
            )
            .map_err(|error| {
                format!(
                    "invalid TypeScript transform option in `{}`: {error}",
                    path.display()
                )
            })?;
            let modes = mode_override.map_or(modes, |mode| vec![mode]);
            let relative_path = path
                .strip_prefix(&test_dir)
                .expect("TypeScript case is inside the cases directory")
                .to_path_buf();
            let has_multiple_units = parsed.test_units.len() > 1;
            for unit in parsed.test_units {
                let Some(lang) = lang_from_path(&unit.name).filter(|lang| lang.is_typescript())
                else {
                    continue;
                };
                let code = Arc::<str>::from(unit.content);
                for &mode in &modes {
                    cases.push(Self {
                        path: path.clone(),
                        relative_path: relative_path.clone(),
                        name: format!("{} [{}]", unit.name, mode.name()),
                        virtual_file_name: has_multiple_units.then(|| unit.name.clone()),
                        code: Arc::clone(&code),
                        mode,
                        source_type: SourceType::from_path(&unit.name),
                        lang,
                    });
                }
            }
        }
        Ok(cases)
    }
}

impl Case for TypeScriptTransformCase {
    const NAME: &'static str = "typescript";
    fn read() -> Vec<Self> {
        Self::read_all(None).expect("failed to read TypeScript transform cases")
    }
    fn path(&self) -> &Path {
        &self.path
    }
    fn relative_path(&self) -> &Path {
        &self.relative_path
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn code(&self) -> &str {
        &self.code
    }
    fn source_type(&self) -> SourceType {
        self.source_type
    }
    fn lang(&self) -> Lang {
        self.lang
    }
    fn expects_error(&self) -> bool {
        false
    }
}

impl TransformCase for TypeScriptTransformCase {
    fn virtual_file_name(&self) -> Option<&str> {
        self.virtual_file_name.as_deref()
    }

    fn options_label(&self) -> Option<&str> {
        Some(self.mode.name())
    }

    fn transformer_options(&self) -> TransformerOptions {
        TransformerOptions {
            typescript: Some(TypeScriptOptions {
                verbatim_module_syntax: self.mode.verbatim_module_syntax(),
                preserve_const_enums: self.mode.preserve_const_enums(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

fn transform_modes(
    verbatim_module_syntax: Option<&str>,
    preserve_const_enums: Option<&str>,
) -> Result<Vec<TypeScriptTransformMode>, String> {
    let verbatim_module_syntax = option_values("verbatimModuleSyntax", verbatim_module_syntax)?;
    let preserve_const_enums = option_values("preserveConstEnums", preserve_const_enums)?;
    let mut modes = Vec::new();
    for &verbatim_module_syntax in &verbatim_module_syntax {
        for &preserve_const_enums in &preserve_const_enums {
            let mode = TypeScriptTransformMode::new(verbatim_module_syntax, preserve_const_enums);
            if !modes.contains(&mode) {
                modes.push(mode);
            }
        }
    }
    Ok(modes)
}

fn option_values(name: &str, option: Option<&str>) -> Result<Vec<bool>, String> {
    let mut values = Vec::new();
    for value in option.unwrap_or("false").split(',') {
        let value = match value.trim().to_ascii_lowercase().as_str() {
            "false" => false,
            "true" => true,
            value => {
                return Err(format!(
                    "`{name}` expected `true` or `false`, found `{value}`"
                ));
            }
        };
        if !values.contains(&value) {
            values.push(value);
        }
    }
    Ok(values)
}
