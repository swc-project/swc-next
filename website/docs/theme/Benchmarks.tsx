import type { CSSProperties } from "react";
import type { Benchmark, BenchmarkReport, Result } from "../benchmark-report";

declare const BENCHMARK_REPORT: BenchmarkReport;

const metrics = [
  {
    key: "time",
    title: "Wall time",
    unit: "ms",
  },
  {
    key: "memory",
    title: "Peak memory",
    unit: "MiB",
  },
] as const;
const benchmarks: Benchmark[] = BENCHMARK_REPORT;
const format = (value: number) =>
  value.toLocaleString("en-US", {
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
const engineName = (name: string) =>
  name.replace(
    /@([a-f0-9]{40})(-dirty)?$/,
    (_, commit: string, dirty: string = "") =>
      `@${commit.slice(0, 8)}${dirty}`,
  );
const engineLabel = (name: string) =>
  name.startsWith("@babel/")
    ? "Babel"
    : name.startsWith("@swc-next/") || name.startsWith("swc-next@")
      ? "SWC Next"
      : "SWC";
const jsTimeMetric = { ...metrics[0], title: "Median best round" } as const;
const metricsFor = (benchmark: Benchmark) =>
  benchmark.type === "js" ? [jsTimeMetric] : metrics;

function baselineFor(results: Result[]): Result {
  const baseline = results.find(
    (result) => engineLabel(result.name) === "SWC Next",
  );
  if (!baseline) throw new Error("Missing SWC Next benchmark baseline");
  return baseline;
}

// Validate during static rendering so invalid data fails the docs build.
for (const benchmark of benchmarks) {
  const results = benchmark.results;
  if (results.length !== (benchmark.type === "js" ? 3 : 2))
    throw new Error(`Unexpected engine count: ${benchmark.id}`);
  baselineFor(results);
  for (const result of results) {
    for (const metric of metricsFor(benchmark)) {
      const statistic = result[metric.key];
      if (statistic === undefined || !Number.isFinite(statistic) || statistic <= 0) {
        throw new Error(`Invalid ${metric.key} summary: ${benchmark.id}`);
      }
    }
  }
}

type Metric = (typeof metrics)[number] | typeof jsTimeMetric;
const ratioFormat = (value: number) =>
  value.toLocaleString("en-US", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });

function statisticFor(result: Result, metric: Metric) {
  const statistic = result[metric.key];
  if (statistic === undefined) throw new Error(`Missing ${metric.key}: ${result.name}`);
  return statistic;
}

function MetricBars({
  results,
  metric,
  maximum,
}: {
  results: Result[];
  metric: Metric;
  maximum: number;
}) {
  const baseline = statisticFor(baselineFor(results), metric);
  return (
    <div className="benchmark-metric" role="group" aria-label={metric.title}>
      {results.map((result, index) => {
        const value = statisticFor(result, metric);
        const relative = value / baseline;
        const label = `${ratioFormat(relative)}×`;
        const absolute = `${format(value)} ${metric.unit}`;
        return (
          <div
            className="benchmark-bar"
            key={result.name}
            aria-label={`${engineLabel(result.name)}: ${label} relative to SWC Next, ${absolute}`}
          >
            <div className="benchmark-bar__track">
              <div
                className={`benchmark-bar__fill benchmark-engine-${index}`}
                style={{ width: `${(relative / maximum) * 100}%` }}
              />
            </div>
            <div className="benchmark-bar__value">
              <strong>{label}</strong>
              <span>({absolute})</span>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function BenchmarkGroup({ title }: { title: string }) {
  const cases = benchmarks.filter((benchmark) => benchmark.title === title);
  if (cases.length === 0) return <p>No results recorded yet.</p>;
  const metrics = metricsFor(cases[0]);
  const results = cases.map((benchmark) => benchmark.results);
  const maxima = metrics.map((metric) =>
    Math.max(
      ...results.flatMap((entries) => {
        const baseline = statisticFor(baselineFor(entries), metric);
        return entries.map(
          (result) => statisticFor(result, metric) / baseline,
        );
      }),
    ),
  );
  return (
    <section className="benchmark-group">
      <figure
        className="benchmark-comparison rp-not-doc"
        aria-label={`${title}: ${metrics.map((metric) => metric.title.toLowerCase()).join(" and ")} relative to SWC Next = 1×`}
      >
        <div className="benchmark-legend">
          {results[0].map((result, index) => (
            <div
              className="benchmark-legend__item"
              key={result.name}
              title={result.name}
            >
              <span className={`benchmark-swatch benchmark-engine-${index}`} />
              <div>
                <strong>{engineLabel(result.name)}</strong>
                <span>{engineName(result.name)}</span>
              </div>
            </div>
          ))}
        </div>
        <div className="benchmark-comparison__scroll rp-scrollbar">
          <div
            className="benchmark-grid"
            style={{ "--benchmark-metric-count": metrics.length } as CSSProperties}
          >
            <div className="benchmark-grid__header">
              <span>Input</span>
              {metrics.map((metric) => (
                <div key={metric.key}>
                  <strong>{metric.title}</strong>
                </div>
              ))}
            </div>
            {cases.map((benchmark, index) => (
              <div className="benchmark-grid__row" key={benchmark.id}>
                <div className="benchmark-input" id={benchmark.id}>
                  <strong>{benchmark.file}</strong>
                </div>
                {metrics.map((metric, metricIndex) => (
                  <MetricBars
                    key={metric.key}
                    metric={metric}
                    results={results[index]}
                    maximum={maxima[metricIndex]}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>
      </figure>
    </section>
  );
}
