#!/usr/bin/env node

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const countFormatter = new Intl.NumberFormat("en-US");

const metricGroups = [
  {
    title: "Callgrind",
    definitions: [
      {
        key: "instructions",
        label: "Instructions",
        tool: "Callgrind",
        summary: "Callgrind",
        events: ["Ir"],
      },
      {
        key: "estimatedCycles",
        label: "Estimated cycles",
        tool: "Callgrind",
        summary: "Callgrind",
        events: ["EstimatedCycles"],
      },
      {
        key: "d1DataMisses",
        label: "D1 data misses",
        tool: "Callgrind",
        summary: "Callgrind",
        events: ["D1mr", "D1mw"],
      },
      {
        key: "llDataMisses",
        label: "LL data misses",
        tool: "Callgrind",
        summary: "Callgrind",
        events: ["DLmr", "DLmw"],
      },
    ],
  },
  {
    title: "DHAT",
    definitions: [
      {
        key: "totalBytes",
        label: "Total bytes",
        tool: "DHAT",
        summary: "Dhat",
        events: ["TotalBytes"],
      },
      {
        key: "totalBlocks",
        label: "Total blocks",
        tool: "DHAT",
        summary: "Dhat",
        events: ["TotalBlocks"],
      },
      {
        key: "peakBytes",
        label: "Peak bytes",
        tool: "DHAT",
        summary: "Dhat",
        events: ["AtTGmaxBytes"],
      },
      {
        key: "peakBlocks",
        label: "Peak blocks",
        tool: "DHAT",
        summary: "Dhat",
        events: ["AtTGmaxBlocks"],
      },
    ],
  },
];

const allDefinitions = metricGroups.flatMap((group) => group.definitions);

function hasOwn(value, key) {
  return Object.prototype.hasOwnProperty.call(value, key);
}

function benchmarkName(summary) {
  const segments = summary.module_path.split("::");
  const functionName = summary.function_name ?? segments.at(-1);
  const groupName = segments.at(-2);

  if (!groupName || groupName === functionName) {
    return functionName;
  }
  return `${groupName}::${functionName}`;
}

function integerCount(metric, side, event, benchmark) {
  if (!metric || typeof metric !== "object" || !hasOwn(metric, "Int")) {
    throw new Error(`Missing ${side} ${event} value for ${benchmark}`);
  }
  if (!Number.isSafeInteger(metric.Int) || metric.Int < 0) {
    throw new Error(`Invalid ${side} ${event} value for ${benchmark}`);
  }
  return metric.Int;
}

function comparisonVariant(values, event, benchmark) {
  if (!values || typeof values !== "object") {
    throw new Error(`Missing ${event} values for ${benchmark}`);
  }
  if (hasOwn(values, "Both")) {
    if (!Array.isArray(values.Both) || values.Both.length !== 2) {
      throw new Error(`Invalid merge/base ${event} values for ${benchmark}`);
    }
    return "both";
  }
  if (hasOwn(values, "Left")) {
    return "new";
  }
  if (hasOwn(values, "Right")) {
    return "removed";
  }
  throw new Error(`Unsupported ${event} values for ${benchmark}`);
}

function percentChange(merge, base) {
  if (base === 0) {
    return merge === 0 ? 0 : Number.POSITIVE_INFINITY;
  }
  return ((merge - base) / base) * 100;
}

function metricComparison(summary, definition, benchmark) {
  const profile = summary.profiles.find(
    (candidate) => candidate.tool === definition.tool,
  );
  if (!profile) {
    return null;
  }

  const metrics = profile.summaries?.total?.summary?.[definition.summary];
  if (!metrics || typeof metrics !== "object") {
    throw new Error(`Missing ${definition.tool} metrics for ${benchmark}`);
  }
  if (definition.events.some((event) => !hasOwn(metrics, event))) {
    return null;
  }

  let comparison;
  let merge = 0;
  let base = 0;
  for (const event of definition.events) {
    const values = metrics[event]?.metrics;
    const eventComparison = comparisonVariant(values, event, benchmark);
    if (comparison && comparison !== eventComparison) {
      throw new Error(
        `Mismatched comparison sides for ${definition.label} in ${benchmark}`,
      );
    }
    comparison = eventComparison;

    if (comparison === "both") {
      merge += integerCount(values.Both[0], "merge", event, benchmark);
      base += integerCount(values.Both[1], "base", event, benchmark);
    } else if (comparison === "new") {
      merge += integerCount(values.Left, "merge", event, benchmark);
    } else {
      base += integerCount(values.Right, "base", event, benchmark);
    }
  }

  return {
    merge: comparison === "removed" ? null : merge,
    base: comparison === "new" ? null : base,
    comparison,
    change: comparison === "both" ? percentChange(merge, base) : null,
  };
}

function summaryRow(summary) {
  if (String(summary.version) !== "6") {
    throw new Error(
      `Unsupported Gungraun summary schema version: ${summary.version}`,
    );
  }
  if (!Array.isArray(summary.profiles)) {
    throw new Error("Gungraun summary is missing profiles");
  }

  const benchmark = benchmarkName(summary);
  const metrics = Object.fromEntries(
    allDefinitions.map((definition) => [
      definition.key,
      metricComparison(summary, definition, benchmark),
    ]),
  );
  if (Object.values(metrics).every((metric) => metric === null)) {
    throw new Error(`No supported metrics for ${benchmark}`);
  }
  return {
    benchmark,
    caseName: summary.id ?? summary.details ?? "default",
    metrics,
  };
}

function compareRows(left, right) {
  return `${left.benchmark}\0${left.caseName}`.localeCompare(
    `${right.benchmark}\0${right.caseName}`,
  );
}

function benchmarkGroup(row) {
  const segments = row.benchmark.split("::");
  return segments.at(-2) ?? "benchmarks";
}

function benchmarkLabel(row, includeBenchmark) {
  const segments = row.benchmark.split("::");
  const functionName = segments.at(-1).replace(/^bench_/, "");
  return includeBenchmark ? `${functionName}/${row.caseName}` : row.caseName;
}

function groupTitle(group) {
  const title = group.replaceAll("_", " ");
  return title.charAt(0).toUpperCase() + title.slice(1);
}

function escapeTableCell(value) {
  return String(value)
    .replaceAll("\\", "/")
    .replaceAll("|", "\\|")
    .replaceAll("`", "'")
    .replace(/\s+/g, " ")
    .trim();
}

function formatSignedShort(value) {
  const magnitude = Math.abs(value);
  const fractionDigits =
    magnitude < 10
      ? 5
      : magnitude < 100
        ? 4
        : magnitude < 1_000
          ? 3
          : magnitude < 10_000
            ? 2
            : magnitude < 100_000
              ? 1
              : 0;
  const sign = value < 0 ? "−" : "+";
  return `${sign}${magnitude.toFixed(fractionDigits)}`;
}

function formatChange(metric) {
  if (metric.comparison === "new") {
    return "New";
  }
  if (metric.comparison === "removed") {
    return "Removed";
  }
  if (metric.change === 0) {
    return "0% ⚪";
  }
  const marker = metric.change < 0 ? "🟢" : "🔴";
  if (!Number.isFinite(metric.change)) {
    return `+∞% ${marker}`;
  }
  return `${formatSignedShort(metric.change)}% ${marker}`;
}

function formatCounts(metric) {
  const merge = metric.merge === null ? "—" : countFormatter.format(metric.merge);
  const base = metric.base === null ? "—" : countFormatter.format(metric.base);
  return `${merge} \\| ${base}`;
}

function groupRows(rows) {
  const groups = new Map();
  for (const row of rows) {
    const group = benchmarkGroup(row);
    const rowsInGroup = groups.get(group) ?? [];
    rowsInGroup.push(row);
    groups.set(group, rowsInGroup);
  }
  return groups;
}

function renderMetricTable(rows, definitions) {
  const benchmarkCount = new Set(rows.map((row) => row.benchmark)).size;
  const includeBenchmark = benchmarkCount > 1;
  const header = definitions.flatMap((definition) => [
    `${definition.label} (head \\| base)`,
    "Change",
  ]);
  const lines = [
    `| Benchmark | ${header.join(" | ")} |`,
    `| --- | ${definitions.map(() => "---: | ---:").join(" | ")} |`,
  ];
  for (const row of rows) {
    const label = escapeTableCell(benchmarkLabel(row, includeBenchmark));
    const cells = definitions.flatMap((definition) => {
      const metric = row.metrics[definition.key];
      return [formatCounts(metric), formatChange(metric)];
    });
    lines.push(`| \`${label}\` | ${cells.join(" | ")} |`);
  }
  return lines.join("\n");
}

function renderGroupTables(group, rows) {
  const sections = [`##### ${groupTitle(group)}`, ""];
  for (const metricGroup of metricGroups) {
    const definitions = metricGroup.definitions.filter((definition) =>
      rows.every((row) => row.metrics[definition.key] !== null),
    );
    if (definitions.length === 0) {
      continue;
    }
    sections.push(`###### ${metricGroup.title}`, "");
    sections.push(renderMetricTable(rows, definitions), "");
  }
  return sections.join("\n");
}

export function parseSummaries(input) {
  const trimmed = input.trim();
  if (!trimmed) {
    return [];
  }
  if (trimmed.startsWith("[")) {
    const summaries = JSON.parse(trimmed);
    if (!Array.isArray(summaries)) {
      throw new Error("Expected an array of Gungraun summaries");
    }
    return summaries;
  }
  if (!trimmed.includes("\n")) {
    return [JSON.parse(trimmed)];
  }
  return trimmed.split(/\r?\n/).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(
        `Invalid Gungraun JSON on line ${index + 1}: ${error.message}`,
      );
    }
  });
}

export async function loadSummaries(inputPath) {
  const inputStat = await stat(inputPath);
  if (inputStat.isFile()) {
    return parseSummaries(await readFile(inputPath, "utf8"));
  }
  if (!inputStat.isDirectory()) {
    throw new Error(`Expected a file or directory: ${inputPath}`);
  }

  const summaries = [];
  const directories = [inputPath];
  for (let index = 0; index < directories.length; index += 1) {
    const directory = directories[index];
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        directories.push(path);
      } else if (entry.isFile() && entry.name === "summary.json") {
        summaries.push(...parseSummaries(await readFile(path, "utf8")));
      }
    }
  }
  return summaries;
}

export function renderSummary(summaries) {
  const rows = summaries.map(summaryRow).sort(compareRows);
  if (rows.length === 0) {
    throw new Error("No benchmark comparisons to summarize");
  }
  const tables = Array.from(groupRows(rows), ([group, rowsInGroup]) =>
    renderGroupTables(group, rowsInGroup),
  ).join("\n");
  return [
    "### Gungraun metric changes",
    "",
    "Exact profiling counts for the head and base revisions. D1 and LL data misses combine reads and writes. Estimated cycles are a cache-simulation metric, not wall-clock time. Markers: 🟢 fewer events, 🔴 more events, ⚪ unchanged.",
    "",
    tables,
  ].join("\n");
}

export function renderFailureSummary(message) {
  const safeMessage = String(message ?? "")
    .replaceAll("`", "'")
    .replace(/\s+/g, " ")
    .trim();
  const detail = safeMessage ? ` Renderer error: ${safeMessage}` : "";
  return [
    "### Gungraun metric changes",
    "",
    `⚠️ **The comparison summary could not be generated.** See the workflow log.${detail}`,
    "",
  ].join("\n");
}

async function main() {
  const inputPath = process.argv[2];
  if (!inputPath) {
    throw new Error("Usage: render-gungraun-summary.mjs <summary path>");
  }
  const summaries = await loadSummaries(inputPath);
  if (summaries.length === 0) {
    throw new Error("No Gungraun summary files found");
  }
  process.stdout.write(renderSummary(summaries));
}

const entryPoint = process.argv[1]
  ? pathToFileURL(process.argv[1]).href
  : undefined;
if (import.meta.url === entryPoint) {
  main().catch((error) => {
    console.error(error);
    process.stdout.write(renderFailureSummary(error.message));
    process.exitCode = 1;
  });
}
