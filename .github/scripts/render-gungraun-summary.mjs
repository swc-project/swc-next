#!/usr/bin/env node

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const countFormatter = new Intl.NumberFormat("en-US");

const primaryMetrics = [
  { key: "d1Data", label: "D1 data misses", events: ["D1mr", "D1mw"] },
  { key: "llData", label: "LL data misses", events: ["DLmr", "DLmw"] },
  {
    key: "estimatedCycles",
    label: "Estimated cycles",
    events: ["EstimatedCycles"],
  },
];

const secondaryMetrics = [
  { key: "i1Read", label: "I1mr", events: ["I1mr"] },
  { key: "llInstructionRead", label: "ILmr", events: ["ILmr"] },
  { key: "instructions", label: "Instructions", events: ["Ir"] },
];

const allMetrics = [...primaryMetrics, ...secondaryMetrics];

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

  const count = metric.Int;
  if (!Number.isSafeInteger(count) || count < 0) {
    throw new Error(`Invalid ${side} ${event} count for ${benchmark}`);
  }
  return count;
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

function metricComparison(metrics, definition, benchmark) {
  let comparison;
  let merge = 0;
  let base = 0;

  for (const event of definition.events) {
    const values = metrics?.[event]?.metrics;
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
  const callgrind = summary.profiles.find(
    (profile) => profile.tool === "Callgrind",
  );
  const metrics = callgrind?.summaries?.total?.summary?.Callgrind;
  if (!metrics || typeof metrics !== "object") {
    throw new Error(`Missing Callgrind metrics for ${benchmark}`);
  }

  return {
    benchmark,
    caseName: summary.id ?? summary.details ?? "default",
    metrics: Object.fromEntries(
      allMetrics.map((definition) => [
        definition.key,
        metricComparison(metrics, definition, benchmark),
      ]),
    ),
  };
}

function compareRows(left, right) {
  const leftKey = `${left.benchmark}\0${left.caseName}`;
  const rightKey = `${right.benchmark}\0${right.caseName}`;
  return leftKey.localeCompare(rightKey);
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
  let fractionDigits;
  if (magnitude < 10) {
    fractionDigits = 5;
  } else if (magnitude < 100) {
    fractionDigits = 4;
  } else if (magnitude < 1_000) {
    fractionDigits = 3;
  } else if (magnitude < 10_000) {
    fractionDigits = 2;
  } else if (magnitude < 100_000) {
    fractionDigits = 1;
  } else {
    fractionDigits = 0;
  }

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
    `${definition.label} (merge \\| base)`,
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
  return [
    `##### ${groupTitle(group)}`,
    "",
    renderMetricTable(rows, primaryMetrics),
    "",
    renderMetricTable(rows, secondaryMetrics),
    "",
  ].join("\n");
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
    const input = await readFile(inputPath, "utf8");
    return parseSummaries(input);
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
        const input = await readFile(path, "utf8");
        summaries.push(...parseSummaries(input));
      }
    }
  }
  return summaries;
}

export function renderSummary(summaries) {
  const rows = summaries.map(summaryRow).sort(compareRows);
  if (rows.length === 0) {
    throw new Error("No cache comparisons to summarize");
  }

  const tables = Array.from(groupRows(rows), ([group, rowsInGroup]) =>
    renderGroupTables(group, rowsInGroup),
  ).join("\n");

  return [
    "### Cache and instruction changes",
    "",
    "Exact Callgrind cache-simulation counts for the merge and base revisions. D1 and LL data misses combine reads and writes. Estimated cycles and instruction counts are profiling metrics, not wall-clock time. Markers: 🟢 fewer events, 🔴 more events, ⚪ unchanged.",
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
    "### Cache and instruction changes",
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
