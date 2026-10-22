import { readFileSync } from "node:fs";

export interface Statistic {
  unit: string;
  count: number;
  mean: number;
  stddev: number;
  min: number;
  max: number;
}
export interface Result {
  name: string;
  summary: { time_wall_clock: Statistic; memory_peak_resident: Statistic };
}
export interface Benchmark {
  id: string;
  title: string;
  type: string;
  operation: string;
  file: string;
  iterations: number;
  runs: number;
  warmup: number;
  processWarmups: number;
  validations: number;
  sessions: { order: string[]; report: { schema_version: number; results: Result[] } }[];
  legacyResults?: Result[];
}

export interface BenchmarkReport {
  formatVersion: number;
  source: string;
  metadata: {
    commit: string;
    generatedAt?: string;
    hyperfineVersion?: string;
    nodeVersion?: string;
    platform?: string;
    architecture?: string;
    cpuModel?: string | null;
    cpuAffinity?: string | null;
  };
  benchmarks: Benchmark[];
}

// Keep raw samples in the source report, outside the browser bundle.
export function loadBenchmarkReport(reportPath = new URL("./data/benchmarks.json", import.meta.url)): BenchmarkReport {
  const report: BenchmarkReport = JSON.parse(readFileSync(reportPath, "utf8"));
  if (report.formatVersion !== 1) throw new Error("Unsupported benchmark report format");
  const summarize = ({ name, summary }: Result): Result => ({
    name,
    summary: { time_wall_clock: summary.time_wall_clock, memory_peak_resident: summary.memory_peak_resident },
  });
  return {
    ...report,
    benchmarks: report.benchmarks.map((benchmark) => ({
      ...benchmark,
      legacyResults: benchmark.legacyResults?.map(summarize),
      sessions: benchmark.sessions.map(({ order, report }) => ({
        order,
        report: { schema_version: report.schema_version, results: report.results.map(summarize) },
      })),
    })),
  };
}
