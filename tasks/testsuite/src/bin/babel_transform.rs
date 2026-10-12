use std::process::ExitCode;
use testsuite::{
    cases::babel::BabelTransformCase, runner::transform::TransformerRunner, transform,
};
fn main() -> ExitCode {
    transform::run::<TransformerRunner, BabelTransformCase>()
}
