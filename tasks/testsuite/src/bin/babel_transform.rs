use std::process::ExitCode;

use testsuite::{
    cases::babel::BabelTransformCase, runner::babel_transform::BabelTransformRunner, transform,
};

fn main() -> ExitCode {
    transform::run::<BabelTransformRunner, BabelTransformCase>()
}
