use std::path::PathBuf;

use jwalk::WalkDir;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::cases::{
    fixtures,
    typescript::{
        lang_from_path, merge_tsconfig_options, parse_test_files_and_symlinks, read_typescript_file,
    },
};

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

#[derive(Debug)]
pub struct TypeScriptTransformUnit {
    pub name: String,
    pub code: String,
    pub lang: Lang,
    pub source_type: SourceType,
}

#[derive(Debug)]
pub struct TypeScriptTransformInput {
    pub relative_path: PathBuf,
    pub modes: Vec<TypeScriptTransformMode>,
    pub units: Vec<TypeScriptTransformUnit>,
}

impl TypeScriptTransformInput {
    pub fn read_all() -> Result<Vec<Self>, String> {
        let source_root = fixtures().join("typescript/tests/cases");
        let mut inputs = Vec::new();

        for suite in ["compiler", "conformance"] {
            let suite_root = source_root.join(suite);
            for entry in WalkDir::new(&suite_root) {
                let entry = entry.map_err(|error| {
                    format!(
                        "failed to enumerate TypeScript inputs in `{}`: {error}",
                        suite_root.display()
                    )
                })?;
                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path();
                let path_text = path.to_string_lossy();
                if !lang_from_path(&path_text).is_some_and(Lang::is_typescript) {
                    continue;
                }
                let relative_path = path
                    .strip_prefix(&source_root)
                    .map_err(|error| {
                        format!(
                            "TypeScript input `{}` is not inside `{}`: {error}",
                            path.display(),
                            source_root.display()
                        )
                    })?
                    .to_path_buf();
                let (modes, units) = read_transform_units(&path)?;
                if units.is_empty() {
                    continue;
                }
                inputs.push(Self {
                    relative_path,
                    modes,
                    units,
                });
            }
        }

        inputs.sort_unstable_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(inputs)
    }
}

fn read_transform_units(
    path: &std::path::Path,
) -> Result<(Vec<TypeScriptTransformMode>, Vec<TypeScriptTransformUnit>), String> {
    let source = read_typescript_file(path).map_err(|error| {
        format!(
            "failed to read TypeScript transform input `{}`: {error}",
            path.display()
        )
    })?;
    let path_text = path.to_string_lossy();
    let mut parsed = parse_test_files_and_symlinks(&source, &path_text);
    merge_tsconfig_options(path, &mut parsed);
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
    Ok((modes, transform_units(parsed.test_units)))
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

fn transform_units(test_units: Vec<super::typescript::TestUnit>) -> Vec<TypeScriptTransformUnit> {
    test_units
        .into_iter()
        .filter_map(|unit| {
            let lang = lang_from_path(&unit.name)?;
            if !lang.is_typescript() {
                return None;
            }
            let source_type = SourceType::from_path(&unit.name);
            Some(TypeScriptTransformUnit {
                name: unit.name,
                code: unit.content,
                lang,
                source_type,
            })
        })
        .collect()
}
