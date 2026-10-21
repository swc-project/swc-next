import type { Benchmark, BenchmarkReport, Result } from "../benchmark-report";

declare const BENCHMARK_REPORT: BenchmarkReport;
const report = BENCHMARK_REPORT;

const metrics = [
  { key: "time_wall_clock", title: "Wall time", unit: "ms", baseUnit: "second", scale: 1000 },
  { key: "memory_peak_resident", title: "Peak memory", unit: "MiB", baseUnit: "byte", scale: 1 / 1048576 },
] as const;
const benchmarks: Benchmark[] = report.benchmarks;
const format = (value: number) => value.toLocaleString("en-US", { maximumFractionDigits: 1, minimumFractionDigits: 1 });
const engineName = (name: string) => name.split(";")[0].replace(/@([a-f0-9]{40})(-dirty)?$/, (_, commit: string, dirty: string = "") => `@${commit.slice(0, 8)}${dirty}`);

function resultsFor(benchmark: Benchmark, session = 0): Result[] {
  const run = benchmark.sessions[session];
  if (run) {
    if (run.report.schema_version !== 2) throw new Error("Unsupported hyperfine JSON schema");
    return run.order.map((name) => {
      const result = run.report.results.find((result) => result.name === name);
      if (!result) throw new Error(`Missing benchmark result: ${name}`);
      return result;
    });
  }
  if (session === 0 && benchmark.legacyResults) return benchmark.legacyResults;
  throw new Error(`Missing benchmark session: ${benchmark.id}`);
}

// Validate during static rendering so invalid exports fail the docs build.
for (const benchmark of benchmarks) {
  for (let session = 0; session < Math.max(1, benchmark.sessions.length); session++) {
    const results = resultsFor(benchmark, session);
    if (results.length !== 2) throw new Error(`Expected two engines: ${benchmark.id}`);
    for (const result of results) {
      for (const metric of metrics) {
        const statistic = result.summary[metric.key];
        if (!statistic || statistic.unit !== metric.baseUnit || statistic.count !== benchmark.runs ||
          ![statistic.mean, statistic.stddev, statistic.min, statistic.max].every((value) => Number.isFinite(value) && value >= 0) ||
          statistic.mean <= 0 || statistic.min > statistic.mean || statistic.mean > statistic.max) {
          throw new Error(`Invalid ${metric.key} summary: ${benchmark.id}`);
        }
      }
    }
  }
}

type Metric = (typeof metrics)[number];
const ratioFormat = (value: number) => value.toLocaleString("en-US", { maximumFractionDigits: 2, minimumFractionDigits: 2 });

function comparison(value: number, baseline: number, metric: Metric) {
  if (metric.key === "memory_peak_resident") return `${ratioFormat(value / baseline)}× memory`;
  return value <= baseline
    ? `${ratioFormat(baseline / value)}× faster`
    : `${ratioFormat(value / baseline)}× slower`;
}

function MetricBars({ results, metric, maximum }: { results: Result[]; metric: Metric; maximum: number }) {
  const baseline = results[1].summary[metric.key].mean;
  return <div className="benchmark-metric" role="group" aria-label={metric.title}>
    {results.map((result, index) => {
      const value = result.summary[metric.key].mean;
      const label = `${format(value * metric.scale)} ${metric.unit}`;
      const relative = index === 0 ? comparison(value, baseline, metric) : null;
      return <div className="benchmark-bar" key={result.name} aria-label={`${index === 0 ? "SWC Next" : "SWC"}: ${label}${relative ? `, ${relative}` : ""}`}>
        <div className="benchmark-bar__track"><div className={`benchmark-bar__fill benchmark-engine-${index}`} style={{ width: `${value / maximum * 100}%` }} /></div>
        <div className="benchmark-bar__value"><strong>{label}</strong>{relative && <span>({relative})</span>}</div>
      </div>;
    })}
  </div>;
}

function Statistics({ results }: { results: Result[] }) {
  return <div className="benchmark-table rp-scrollbar">
    <table>
      <thead><tr><th>Engine</th><th>Metric</th><th>Mean ± σ</th><th>Min</th><th>Max</th></tr></thead>
      <tbody>{results.flatMap((result) => metrics.map((metric, index) => {
        const statistic = result.summary[metric.key];
        const display = (value: number) => `${format(value * metric.scale)} ${metric.unit}`;
        return <tr key={`${result.name}-${metric.key}`}>
          {index === 0 && <th rowSpan={2} scope="rowgroup" title={result.name}>
            <span>{engineName(result.name).slice(0, engineName(result.name).lastIndexOf("@"))}</span>
            <span className="benchmark-version">{engineName(result.name).slice(engineName(result.name).lastIndexOf("@") + 1)}</span>
          </th>}
          <td>{metric.title}</td><td>{display(statistic.mean)} ± {display(statistic.stddev)}</td><td>{display(statistic.min)}</td><td>{display(statistic.max)}</td>
        </tr>;
      }))}</tbody>
    </table>
  </div>;
}

export function BenchmarkGroup({ title }: { title: string }) {
  const cases = benchmarks.filter((benchmark) => benchmark.title === title);
  const results = cases.map((benchmark) => resultsFor(benchmark));
  const maxima = metrics.map((metric) => Math.max(...results.flatMap((pair) => pair.map((result) => result.summary[metric.key].mean))));
  return <section className="benchmark-group">
    <figure className="benchmark-comparison rp-not-doc" aria-label={`${title}: wall time and peak memory compared with SWC`}>
      <div className="benchmark-legend">
        {results[0].map((result, index) => <div className="benchmark-legend__item" key={result.name} title={result.name}>
          <span className={`benchmark-swatch benchmark-engine-${index}`} />
          <div><strong>{index === 0 ? "SWC Next" : "SWC"}</strong><span>{engineName(result.name)}</span></div>
        </div>)}
      </div>
      <div className="benchmark-comparison__scroll rp-scrollbar">
        <div className="benchmark-grid">
          <div className="benchmark-grid__header"><span>Input</span>{metrics.map((metric) => <div key={metric.key}><strong>{metric.title}</strong><span>vs SWC</span></div>)}</div>
          {cases.map((benchmark, index) => <div className="benchmark-grid__row" key={benchmark.id}>
            <div className="benchmark-input" id={benchmark.id}><strong>{benchmark.file}</strong><span>{benchmark.iterations.toLocaleString("en-US")} {benchmark.type === "js" ? "parses" : "iterations"} per process</span></div>
            {metrics.map((metric, metricIndex) => <MetricBars key={metric.key} metric={metric} results={results[index]} maximum={maxima[metricIndex]} />)}
          </div>)}
        </div>
      </div>
      <figcaption>Mean values; lower is better. Bar lengths share a scale within each metric in this group.</figcaption>
    </figure>
    <details className="benchmark-statistics"><summary>Detailed statistics</summary>
      {cases.map((benchmark, index) => <div className="benchmark-case" key={benchmark.id}>
        <h3>{benchmark.file}</h3>
        <p className="benchmark-method">{benchmark.runs} runs per engine · {benchmark.iterations.toLocaleString("en-US")} {benchmark.type === "js" ? "parses" : "iterations"} per process{benchmark.type === "js" ? ` + ${benchmark.validations} validation + ${benchmark.processWarmups} warmups` : ` · ${benchmark.warmup} warmup runs`}</p>
        <Statistics results={results[index]} />
        {benchmark.sessions.length > 1 && <details><summary>Reverse-order verification</summary><Statistics results={resultsFor(benchmark, 1)} /></details>}
      </div>)}
    </details>
  </section>;
}
