use std::path::Path;

use jwalk::WalkDir;
use rayon::prelude::*;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::util::crate_root;

use super::{Case, test262::Test262Case};

/// Project-owned regressions sharing Test262 metadata, but not its suite identity.
pub struct LocalCase(Test262Case);

impl Case for LocalCase {
    const NAME: &'static str = "local";

    fn read() -> Vec<Self> {
        WalkDir::new(crate_root().join("fixtures-manual/parser"))
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                let supported = path
                    .extension()
                    .is_some_and(|ext| ext == "js" || ext == "ts");
                (entry.file_type().is_file() && supported).then_some(path)
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .flat_map(Test262Case::read_file)
            .map(Self)
            .collect()
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn code(&self) -> &str {
        self.0.code()
    }

    fn source_type(&self) -> SourceType {
        self.0.source_type()
    }

    fn lang(&self) -> Lang {
        Lang::from_path(&self.path().to_string_lossy())
    }

    fn expects_error(&self) -> bool {
        self.0.expects_error()
    }
}
