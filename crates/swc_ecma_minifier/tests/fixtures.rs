use std::path::Path;

use libtest_mimic::{Arguments, Trial};

mod support;

fn main() {
    let args = Arguments::from_args();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cases = support::discover(&root).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let tests = cases
        .into_iter()
        .map(|path| {
            let name = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            Trial::test(name, move || {
                support::run_fixture(&path).map_err(Into::into)
            })
        })
        .collect();
    libtest_mimic::run(&args, tests).exit();
}
