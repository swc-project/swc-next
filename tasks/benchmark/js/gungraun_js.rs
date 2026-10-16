use gungraun::{
    Bench, BinaryBenchmark, BinaryBenchmarkConfig, Callgrind, Command, EntryPoint, EventKind,
    FlamegraphConfig, binary_benchmark_group, main,
};

binary_benchmark_group!(
    name = js_api,
    benchmarks = |group: &mut BinaryBenchmarkGroup| {
        let harness = std::env::var("JS_BENCH_HARNESS").expect("JS_BENCH_HARNESS is required");
        let entry = std::env::var("JS_BENCH_ENTRY").expect("JS_BENCH_ENTRY is required");
        let node = std::env::var("JS_BENCH_NODE").expect("JS_BENCH_NODE is required");
        let filter = std::env::var("JS_BENCH_FILTER").unwrap_or_default();
        let exact_case = std::env::var("JS_BENCH_CASE").ok();
        let mut selected = 0;
        for operation in ["parse", "print", "lang", "source_type"] {
            let cases: &[&str] = if matches!(operation, "lang" | "source_type") {
                &["js", "ts", "tsx", "mjs", "cjs", "unknown"]
            } else {
                &[
                    "underscore-1.5.2.js",
                    "jquery-1.9.1.js",
                    "astro-config.ts",
                    "cal.com.tsx",
                ]
            };
            let mut benchmark = BinaryBenchmark::new(operation);
            for case in cases {
                let identity = format!("{operation}/{case}");
                if !identity.contains(&filter)
                    || exact_case.as_ref().is_some_and(|value| value != &identity)
                {
                    continue;
                }
                selected += 1;
                // Three fresh processes, with stable identities on both revisions.
                for trial in 1..=3 {
                    benchmark.bench(
                        Bench::new(format!("{}_trial_{trial}", case.replace(['.', '-'], "_")))
                            .command(
                                Command::new(&node)
                                    .args([
                                        "--jitless",
                                        "--predictable",
                                        "--expose-gc",
                                        "--random-seed=42",
                                    ])
                                    .arg(format!("{harness}/runner.mjs"))
                                    .args([&entry, operation, case])
                                    .env("JS_BENCH_HARNESS", &harness),
                            ),
                    );
                }
            }
            if !benchmark.benches.is_empty() {
                group.binary_benchmark(benchmark);
            }
        }
        assert!(selected > 0, "JS_BENCH_FILTER matched no cases");
    }
);

main!(
    config = BinaryBenchmarkConfig::default().tool(
        Callgrind::with_args([
            "--instr-atstart=no",
            "--collect-atstart=no",
            "--cache-sim=no",
            "--branch-sim=no",
            "--smc-check=all-non-file",
        ])
        .entry_point(EntryPoint::None)
        .format([EventKind::Ir])
        .flamegraph(FlamegraphConfig::default()),
    ),
    binary_benchmark_groups = js_api
);
