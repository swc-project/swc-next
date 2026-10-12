use std::process::ExitCode;
use testsuite::{
    cases::swc_transform::SwcTransformCase, runner::transform::TransformerRunner, transform,
};
fn main() -> ExitCode {
    transform::run::<TransformerRunner, SwcTransformCase>()
}
