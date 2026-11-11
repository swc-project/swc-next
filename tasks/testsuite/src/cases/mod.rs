use std::path::{Path, PathBuf};

use swc_core::ecma::transforms::base::assumptions::Assumptions;
use swc_next_core::js::{JsTransformApiOptions, transformer::JsAssumptions};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_minifier::MinifierOptions;

use crate::util::crate_root;

pub mod babel;
pub mod babel_parser;
pub mod minifier;
pub mod swc_resolver;
pub mod swc_transform;
pub mod test262;
pub mod typescript;

pub trait Case: Sized + Sync {
    const NAME: &'static str;

    fn read() -> Vec<Self>;
    fn path(&self) -> &Path;
    fn name(&self) -> &str;
    fn code(&self) -> &str;
    fn source_type(&self) -> SourceType;
    fn expected_source_type(&self) -> Option<SourceType> {
        None
    }
    fn lang(&self) -> Lang;
    fn expects_error(&self) -> bool;
    /// Whether a case expected to succeed also asserts that semantic analysis
    /// produces no early-error diagnostics.
    fn asserts_semantic_success(&self) -> bool {
        true
    }

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

/// Fixture adapters provide the configuration consumed by the shared transform pipeline.
pub trait TransformCase: Case {
    fn options(&self) -> JsTransformApiOptions;

    /// Identifies the input unit when a fixture contains multiple virtual files.
    fn virtual_file_name(&self) -> Option<&str> {
        None
    }

    fn options_label(&self) -> Option<&str> {
        None
    }

    fn supports_options(&self) -> bool {
        true
    }

    /// An absent oracle requests JavaScript output validation and transform idempotence.
    fn expected_output(&self) -> Option<&str> {
        None
    }
}

/// Fixture adapters for the minifier runner.
pub trait MinifierCase: Case {
    fn minifier_options(&self) -> MinifierOptions;

    /// Output of the reference minifier. An absent oracle skips the case.
    fn expected_output(&self) -> Option<&str>;
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}

pub(super) fn assumptions(options: Assumptions) -> JsAssumptions {
    JsAssumptions {
        array_like_is_iterable: Some(options.array_like_is_iterable),
        constant_reexports: Some(options.constant_reexports),
        constant_super: Some(options.constant_super),
        enumerable_module_meta: Some(options.enumerable_module_meta),
        ignore_function_length: Some(options.ignore_function_length),
        ignore_function_name: Some(options.ignore_function_name),
        ignore_to_primitive_hint: Some(options.ignore_to_primitive_hint),
        iterable_is_array: Some(options.iterable_is_array),
        mutable_template_object: Some(options.mutable_template_object),
        no_class_calls: Some(options.no_class_calls),
        no_document_all: Some(options.no_document_all),
        no_incomplete_ns_import_detection: Some(options.no_incomplete_ns_import_detection),
        no_new_arrows: Some(options.no_new_arrows),
        object_rest_no_symbols: Some(options.object_rest_no_symbols),
        private_fields_as_properties: Some(options.private_fields_as_properties),
        pure_getters: Some(options.pure_getters),
        set_class_methods: Some(options.set_class_methods),
        set_computed_properties: Some(options.set_computed_properties),
        set_public_class_fields: Some(options.set_public_class_fields),
        set_spread_properties: Some(options.set_spread_properties),
        skip_for_of_iterator_closing: Some(options.skip_for_of_iterator_closing),
        super_is_callable_constructor: Some(options.super_is_callable_constructor),
    }
}
