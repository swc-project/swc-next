use crate::{AppArgs, cases::Case, suite::TestResult};

pub mod parser;

pub trait Runner {
    const NAME: &'static str;

    fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult>;
}
