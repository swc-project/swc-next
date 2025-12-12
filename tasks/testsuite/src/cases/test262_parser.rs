use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::cases::{Case, fixtures};

pub struct Test262ParserCase {
    path: PathBuf,
    code: String,
    should_fail: bool,
}

impl Test262ParserCase {
    pub fn read() -> Vec<Self> {
        let test_path = fixtures().join("test262-parser-tests");
        let pass_cases = read_dir(test_path.join("pass")).unwrap();
        let pass_explicit_cases = read_dir(test_path.join("pass-explicit")).unwrap();
        let fail_cases = read_dir(test_path.join("fail")).unwrap();

        let mut cases = Vec::new();
        for pass_case in pass_cases.chain(pass_explicit_cases) {
            let pass_case = pass_case.unwrap();
            let path = pass_case.path();
            let code = std::fs::read_to_string(&path).unwrap();
            cases.push(Self {
                path,
                code,
                should_fail: false,
            });
        }

        for fail_case in fail_cases {
            let fail_case = fail_case.unwrap();
            let path = fail_case.path();
            let code = std::fs::read_to_string(&path).unwrap();
            cases.push(Self {
                path,
                code,
                should_fail: true,
            });
        }

        cases
    }
}

impl Case for Test262ParserCase {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }

    fn should_ignore(&self) -> bool {
        (!self.should_fail() && IGNORED_PASS_TESTS.contains(&self.filename().as_str()))
            || (self.should_fail() && IGNORED_ERROR_TESTS.contains(&self.filename().as_str()))
    }
}

const IGNORED_PASS_TESTS: &[&str] = &[
    // Temporalily ignored
    "431ecef8c85d4d24.js",
    "8386fbff927a9e0e.js",
    "5654d4106d7025c2.js",
    // Stack size (Stupid parens)
    "6b5e7e125097d439.js",
    "714be6d28082eaa7.js",
    "882910de7dd1aef9.js",
    "dd3c63403db5c06e.js",
    // Static constructor
    "dcc5609dcc043200.js",
    "88d42455ac933ef5.js",
    // Wrong tests (variable name or value is different)
    "0339fa95c78c11bd.js",
    "0426f15dac46e92d.js",
    "0b4d61559ccce0f9.js",
    "0f88c334715d2489.js",
    "1093d98f5fc0758d.js",
    "15d9592709b947a0.js",
    "2179895ec5cc6276.js",
    "247a3a57e8176ebd.js",
    "441a92357939904a.js",
    "47f974d6fc52e3e4.js",
    "4e1a0da46ca45afe.js",
    "5829d742ab805866.js",
    "589dc8ad3b9aa28f.js",
    "598a5cedba92154d.js",
    "72d79750e81ef03d.js",
    "7788d3c1e1247da9.js",
    "7b72d7b43bedc895.js",
    "7dab6e55461806c9.js",
    "82c827ccaecbe22b.js",
    "87a9b0d1d80812cc.js",
    "8c80f7ee04352eba.js",
    "96f5d93be9a54573.js",
    "988e362ed9ddcac5.js",
    "9bcae7c7f00b4e3c.js",
    "a8a03a88237c4e8f.js",
    "ad06370e34811a6a.js",
    "b0fdc038ee292aba.js",
    "b62c6dd890bef675.js",
    "cb211fadccb029c7.js",
    "ce968fcdf3a1987c.js",
    "db3c01738aaf0b92.js",
    "e1387fe892984e2b.js",
    "e71c1d5f0b6b833c.js",
    "e8ea384458526db0.js",
    // We don't implement Annex B fully.
    "1c1e2a43fe5515b6.js",
    "3dabeca76119d501.js",
    "52aeec7b8da212a2.js",
    "59ae0289778b80cd.js",
    "a4d62a651f69d815.js",
    "c06df922631aeabc.js",
    // Wrong test - strict mode
    "8f8bfb27569ac008.js",
    "ce569e89a005c02a.js",
    // Unicode 14 vs 15
    "046a0bb70d03d0cc.js",
    "08a39e4289b0c3f3.js",
    "300a638d978d0f2c.js",
    "44f31660bd715f05.js",
];

const IGNORED_ERROR_TESTS: &[&str] = &[
    // pass in script. error in module.
    "e3fbcf63d7e43ead.js",
    // Old (wrong) tests
    "569a2c1bad3beeb2.js",
    "3b6f737a4ac948a8.js",
    "829d9261aa6cd22c.js",
    "b03ee881dce1a367.js",
    "cb92787da5075fd1.js",
    "f0f498d6ae70038f.js",
    // Wrong tests
    "0d5e450f1da8a92a.js",
    "346316bef54d805a.js",
    "976b6247ca78ab51.js",
    "ae0a7ac275bc9f5c.js",
    "748656edbfb2d0bb.js",
    "79f882da06f88c9f.js",
    "d28e80d99f819136.js",
    "92b6af54adef3624.js",
    "ef2d369cccc5386c.js",
    // Temporarily ignore tests for using octal escape before use strict
    "147fa078a7436e0e.js",
    "15a6123f6b825c38.js",
    "3bc2b27a7430f818.js",
    // Temporarily ignored
    "2fa321f0374c7017.js",
    "3dbb6e166b14a6c0.js",
    "66e383bfd18e66ab.js",
    "78c215fabdf13bae.js",
    "bf49ec8d96884562.js",
    "e4a43066905a597b.js",
    "98204d734f8c72b3.js",
    "ef81b93cf9bdb4ec.js",
    // Enabled in SWC but with `verify` feature
    "03d13b6c40f6aaea.js",
    "1a5b0dfa9fde985d.js",
    "a38011d2c010999e.js",
    "4e885526e8dfaa12.js",
    "3d3e6ce2b81a224d.js",
    "08bafe059b17ac92.js",
    "bcde05eea9466dfd.js",
    "c0ad1c20e662c8ed.js",
    "dc5864c9096ad0a8.js",
    "4ef1d6ca8eceb313.js",
    "0ebf57bd8c051d27.js",
];
