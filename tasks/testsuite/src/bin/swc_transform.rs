use std::process::ExitCode;

use testsuite::{
    cases::swc_transform::SwcTransformCase, runner::swc_transform::SwcTransformRunner, transform,
};

fn main() -> ExitCode {
    transform::run::<SwcTransformRunner, SwcTransformCase>()
}
