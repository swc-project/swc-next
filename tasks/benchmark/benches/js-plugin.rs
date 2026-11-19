use std::{
    env,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};

struct PluginWorker {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    bytes: u64,
}

impl PluginWorker {
    fn new(suite: &str, plugin: &str) -> Self {
        let directory = env::var_os("JS_PLUGIN_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::current_dir().expect("benchmark working directory is required")
            });
        let runner = if directory.join("js/js-plugin.mjs").is_file() {
            directory.join("js/js-plugin.mjs")
        } else {
            directory.join("tasks/benchmark/js/js-plugin.mjs")
        };
        let node = env::var("JS_PLUGIN_NODE").unwrap_or_else(|_| "node".into());
        let mut child = Command::new(node)
            .arg(runner)
            .arg(plugin)
            .arg(suite)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("build @swc-next/core and install workspace dependencies before benchmarking");
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut worker = Self {
            child,
            input,
            output,
            bytes: 0,
        };
        let ready = worker.read_line();
        let mut fields = ready.split_whitespace();
        assert_eq!(
            fields.next(),
            Some("ready"),
            "Node benchmark failed during preflight"
        );
        let mode = fields.next().unwrap();
        let files = fields.next().unwrap();
        worker.bytes = fields.next().unwrap().parse().unwrap();
        if let Some(cold) = fields.next() {
            let cold: u64 = cold
                .parse()
                .expect("Node benchmark must report cold plugin nanoseconds");
            eprintln!(
                "{suite}/{plugin}: {mode}, {files} files, {} bytes, cold plugin {:.3} ms",
                worker.bytes,
                Duration::from_nanos(cold).as_secs_f64() * 1000.0
            );
        } else {
            eprintln!(
                "{suite}/{plugin}: {mode}, {files} files, {} bytes",
                worker.bytes
            );
        }
        worker
    }

    fn read_line(&mut self) -> String {
        let mut line = String::new();
        assert!(
            self.output.read_line(&mut line).unwrap() > 0,
            "Node benchmark exited unexpectedly"
        );
        line.trim().to_owned()
    }

    fn measure(&mut self, iterations: u64, phase: &str) -> Duration {
        writeln!(self.input, "{iterations} {phase}").unwrap();
        self.input.flush().unwrap();
        let elapsed: u64 = self
            .read_line()
            .parse()
            .expect("Node benchmark must return nanoseconds");
        Duration::from_nanos(elapsed)
    }
}

impl Drop for PluginWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn bench_plugins(c: &mut Criterion) {
    bench_suite(c, "js-plugin");
    bench_suite(c, "js-plugin2");
}

fn bench_suite(c: &mut Criterion, suite: &str) {
    let mut group = c.benchmark_group(suite);
    // Every phase runs the complete pipeline. Keep sample batches the same
    // size instead of increasing their iteration counts throughout the run.
    group.sampling_mode(SamplingMode::Flat);
    for plugin in [
        "formatjs",
        "import-meta-env",
        "react-remove-properties",
        "ident-length",
        "ident-rename",
    ] {
        // The worker warms the full dataset before returning its ready message.
        // One process per plugin preserves JIT feedback across all three phases.
        let mut worker = None;
        for phase in ["parse", "plugin", "print"] {
            if plugin == "ident-length" && phase == "print" {
                continue;
            }
            group.bench_function(format!("{plugin}/{phase}"), |b| {
                let worker = worker.get_or_insert_with(|| PluginWorker::new(suite, plugin));
                b.iter_custom(|iterations| worker.measure(iterations, phase));
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(10))
        .measurement_time(Duration::from_millis(200));
    targets = bench_plugins
}
criterion_main!(benches);
