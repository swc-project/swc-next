import { readFileSync } from "node:fs";

export interface Result {
  name: string;
  /** Native mean or JS median of trial bests, in milliseconds per operation. */
  time: number;
  /** Native process peak RSS in MiB. */
  memory?: number;
}

export interface Benchmark {
  id: string;
  title: string;
  type: "native" | "js";
  file: string;
  results: Result[];
}

export type BenchmarkReport = Benchmark[];

export function loadBenchmarkReport(reportPath = new URL("./data/benchmarks.json", import.meta.url)): BenchmarkReport {
  const report: BenchmarkReport = JSON.parse(readFileSync(reportPath, "utf8"));
  if (!Array.isArray(report)) throw new Error("Unsupported benchmark report format");
  return report;
}
