pub struct SourceMap<'a> {
    pub(crate) version: u8,
    pub(crate) file: Option<&'a [u8]>,
    pub(crate) source_root: Option<&'a [u8]>,
    pub(crate) sources: &'a [&'a [u8]],
    pub(crate) sources_content: Option<&'a [Option<&'a [u8]>]>,
    pub(crate) names: &'a [&'a [u8]],
    pub(crate) mappings: &'a [u8],
}

pub struct SourceMapBuilder<'a> {
    source: &'a [u8],
}

impl<'a> SourceMapBuilder<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    pub fn build(self) -> SourceMap<'a> {
        todo!()
    }
}
