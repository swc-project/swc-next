#!/usr/bin/env node

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const countFormatter = new Intl.NumberFormat("en-US");
const changeFormatter = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 3,
  useGrouping: false,
});

function hasOwn(value, key) {
  return Object.prototype.hasOwnProperty.call(value, key);
}

function parseDiff(value) {
  if (value === "inf") {
    return Number.POSITIVE_INFINITY;
  }
  if (value === "-inf") {
    return Number.NEGATIVE_INFINITY;
  }

  const parsed = Number(value);
  if (Number.isNaN(parsed)) {
    throw new Error(`Invalid instruction difference: ${value}`);
  }
  return parsed;
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

function integerMetricValue(value, name, side, benchmark) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`Invalid ${side} ${name} value for ${benchmark}`);
  }
  return value;
}

function metricComparison(metrics, event, name, benchmark) {
  const metric = metrics?.[event];
  const values = metric?.values;
  if (!values || typeof values !== "object") {
    throw new Error(`Missing ${name} values for ${benchmark}`);
  }

  if (hasOwn(values, "new") && hasOwn(values, "old")) {
    const compareValue = integerMetricValue(values.new, name, "compare", benchmark);
    const baseValue = integerMetricValue(values.old, name, "base", benchmark);
    return {
      compareValue,
      baseValue,
      comparison: "both",
      change: parseDiff(String(metric.change?.diff_pct)),
    };
  }

  if (hasOwn(values, "new")) {
    const compareValue = integerMetricValue(values.new, name, "compare", benchmark);
    return {
      compareValue,
      baseValue: null,
      comparison: "new",
      change: null,
    };
  }

  if (hasOwn(values, "old")) {
    const baseValue = integerMetricValue(values.old, name, "base", benchmark);
    return {
      compareValue: null,
      baseValue,
      comparison: "removed",
      change: null,
    };
  }

  throw new Error(`Unsupported ${name} values for ${benchmark}`);
}

function summaryRows(summary) {
  if (String(summary.version) !== "7") {
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
  const profile = callgrind?.data;
  if (!profile || !Array.isArray(profile.parts)) {
    throw new Error(`Missing Callgrind profile for ${benchmark}`);
  }

  const caseName = summary.id ?? summary.details ?? "default";
  const row = (label, metrics) => {
    const instructions = metricComparison(
      metrics,
      "Ir",
      "instruction count",
      benchmark,
    );
    return {
      benchmark,
      caseName: label ? `${caseName}/${label}` : caseName,
      compareInstructions: instructions.compareValue,
      baseInstructions: instructions.baseValue,
      comparison: instructions.comparison,
      change: instructions.change,
    };
  };
  const total = profile.total?.metrics;
  if (profile.parts.length <= 1) {
    return [row(null, total)];
  }

  const parts = profile.parts.map((part) => {
    const info = part.tool_run.new ?? part.tool_run.old;
    return row(`Part ${info.part}`, part.metrics);
  });
  parts.push(row("Total", total));
  return parts;
}

function compareRows(left, right) {
  const leftKey = `${left.benchmark}\0${left.caseName}`;
  const rightKey = `${right.benchmark}\0${right.caseName}`;
  if (leftKey < rightKey) {
    return -1;
  }
  if (leftKey > rightKey) {
    return 1;
  }
  return 0;
}

function benchmarkGroup(row) {
  const segments = row.benchmark.split("::");
  return segments.at(-2) ?? "benchmarks";
}

function benchmarkLabel(row, includeBenchmark) {
  const segments = row.benchmark.split("::");
  const functionName = segments.at(-1).replace(/^bench_/, "");
  if (includeBenchmark) {
    return `${functionName}/${row.caseName}`;
  }
  return row.caseName;
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

function formatChange(row) {
  if (row.comparison === "new") {
    return "New";
  }
  if (row.comparison === "removed") {
    return "Removed";
  }
  if (changeFormatter.format(Math.abs(row.change)) === "0") {
    return "0% ⚪";
  }

  const marker = row.inconclusive ? "⚪" : row.change < 0 ? "🟢" : "🔴";
  if (!Number.isFinite(row.change)) {
    const change = row.change < 0 ? "−∞" : "+∞";
    return `${change}% ${marker}`;
  }
  return `${formatSignedShort(row.change)}% ${marker}`;
}

function formatSignedShort(value) {
  const sign = value < 0 ? "−" : "+";
  return `${sign}${changeFormatter.format(Math.abs(value))}`;
}

function formatInstructionCount(value) {
  return value === null
    ? "—"
    : countFormatter.format(Math.trunc(value / 1_000));
}

function formatPair(left, right, formatter) {
  return `${formatter(left)} / ${formatter(right)}`;
}

function groupRows(rows) {
  const groups = new Map();
  for (const row of rows) {
    const group = benchmarkGroup(row);
    const groupRows = groups.get(group) ?? [];
    groupRows.push(row);
    groups.set(group, groupRows);
  }
  return groups;
}

export function renderGroupTable(group, rows) {
  const benchmarkCount = new Set(rows.map((row) => row.benchmark)).size;
  const includeBenchmark = benchmarkCount > 1;
  const lines = [
    `##### ${groupTitle(group)}`,
    "",
    "| Benchmark | Ir (K) | Change |",
    "| --- | ---: | ---: |",
  ];
  for (const row of rows) {
    const label = escapeTableCell(benchmarkLabel(row, includeBenchmark));
    const instructions = formatPair(
      row.compareInstructions,
      row.baseInstructions,
      formatInstructionCount,
    );
    lines.push(
      `| \`${label}\` | ${instructions} | ${formatChange(row)} |`,
    );
  }
  lines.push("");
  return lines.join("\n");
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
  const rows = summaries.flatMap(summaryRows).sort(compareRows);
  if (rows.length === 0) {
    throw new Error("No Callgrind comparisons to summarize");
  }

  const groups = groupRows(rows);
  const tables = Array.from(groups, ([group, rowsInGroup]) =>
    renderGroupTable(group, rowsInGroup),
  ).join("\n");

  return [
    "### Callgrind metric changes",
    "",
    "Callgrind instruction counts (Ir) for the compare and base revisions. Values are shown as compare / base in K (1,000), truncated to whole thousands. Multi-part profiles use the part numbers from Gungraun and include an aggregated Total. New and Removed identify benchmarks that exist on only one side; the missing value is shown as a dash. Percentages are rounded to at most three decimal places. Lower is better: 🟢 improvement, 🔴 regression, ⚪ unchanged.",
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
    "### Callgrind metric changes",
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
