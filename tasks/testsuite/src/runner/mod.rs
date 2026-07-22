use crate::{cases::Case, suite::TestResult};

pub mod codegen;
pub mod parser;

pub trait Runner {
    const NAME: &'static str;

    fn run<C: Case>(cases: &[C], debug: bool) -> Vec<TestResult>;
}
