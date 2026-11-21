/// SWC's `async_to_generator::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AsyncToGeneratorOptions {
    /// Allow lowering to change a function's observable `length`.
    pub ignore_function_length: bool,
}
