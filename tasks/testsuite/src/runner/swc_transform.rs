use crate::{cases::swc_transform::SwcTransformCase, suite::TestResult};

use super::{Runner, babel_transform::run_transform};

pub struct SwcTransformRunner;

impl Runner<SwcTransformCase> for SwcTransformRunner {
    const NAME: &'static str = "swc_transform";

    fn should_run(case: &SwcTransformCase) -> bool {
        !case.requires_class_properties
    }

    fn run(case: &SwcTransformCase) -> TestResult {
        run_transform(case, Self::NAME)
    }
}
