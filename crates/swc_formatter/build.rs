use std::{collections::BTreeSet, env, fs, path::Path};

fn collect(root: &Path, directory: &Path, tests: &mut Vec<String>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(root, &path, tests);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension,
                    "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts"
                )
            })
        {
            tests.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
            );
        }
    }
}

fn main() {
    let root = Path::new("tests/fixtures");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut tests = Vec::new();
    collect(root, root, &mut tests);
    tests.sort();
    let mut names = BTreeSet::new();
    let mut generated = String::new();
    for path in tests {
        let suffix: String = path
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let name = format!("fixture_{suffix}");
        assert!(
            names.insert(name.clone()),
            "duplicate fixture test name: {name}"
        );
        generated.push_str(&format!(
            "#[test]\nfn {name}() {{ fixtures::test_file({path:?}); }}\n"
        ));
    }
    let destination = Path::new(&env::var_os("OUT_DIR").unwrap()).join("formatter_fixtures.rs");
    fs::write(destination, generated).unwrap();
}
