#!/usr/bin/env node

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const instructionFormatter = new Intl.NumberFormat("en-US");

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

function instructionCount(metric, side, benchmark) {
  if (!metric || typeof metric !== "object" || !hasOwn(metric, "Int")) {
    throw new Error(`Missing ${side} instruction value for ${benchmark}`);
  }

  const count = metric.Int;
  if (!Number.isSafeInteger(count) || count < 0) {
    throw new Error(`Invalid ${side} instruction count for ${benchmark}`);
  }
  return count;
}

function instructionComparison(metrics, benchmark) {
  const instruction = metrics?.Ir;
  const values = instruction?.metrics;
  if (!values || typeof values !== "object") {
    throw new Error(`Missing instruction values for ${benchmark}`);
  }

  if (hasOwn(values, "Both")) {
    const pair = values.Both;
    if (!Array.isArray(pair) || pair.length !== 2) {
      throw new Error(`Invalid merge/base instruction values for ${benchmark}`);
    }

    const mergeInstructions = instructionCount(pair[0], "merge", benchmark);
    const baseInstructions = instructionCount(pair[1], "base", benchmark);
    return {
      mergeInstructions,
      baseInstructions,
      comparison: "both",
      change: parseDiff(String(instruction.diffs?.diff_pct)),
    };
  }

  if (hasOwn(values, "Left")) {
    const mergeInstructions = instructionCount(
      values.Left,
      "merge",
      benchmark,
    );
    return {
      mergeInstructions,
      baseInstructions: null,
      comparison: "new",
      change: null,
    };
  }

  if (hasOwn(values, "Right")) {
    const baseInstructions = instructionCount(values.Right, "base", benchmark);
    return {
      mergeInstructions: null,
      baseInstructions,
      comparison: "removed",
      change: null,
    };
  }

  throw new Error(`Unsupported instruction values for ${benchmark}`);
}

function summaryRows(summary) {
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
  const profile = callgrind?.summaries;
  if (!profile || !Array.isArray(profile.parts)) {
    throw new Error(`Missing Callgrind profile for ${benchmark}`);
  }

  const caseName = summary.id ?? summary.details ?? "default";
  const row = (label, metrics) => ({
    benchmark,
    caseName: label ? `${caseName}/${label}` : caseName,
    ...instructionComparison(metrics, benchmark),
  });
  const total = profile.total?.summary?.Callgrind;
  if (profile.parts.length <= 1) {
    return [row(null, total)];
  }

  const parts = profile.parts.map((part) => {
    const info = part.details.Left ?? part.details.Right ?? part.details.Both[0];
    return row(`Part ${info.part}`, part.metrics_summary?.Callgrind);
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
  if (row.change === 0) {
    return "0% ⚪";
  }

  const marker = row.change < 0 ? "🟢" : "🔴";
  if (!Number.isFinite(row.change)) {
    const change = row.change < 0 ? "−∞" : "+∞";
    return `${change}% ${marker}`;
  }
  return `${formatSignedShort(row.change)}% ${marker}`;
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

function formatInstructionCount(value) {
  return value === null ? "—" : instructionFormatter.format(value);
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

function renderGroupTable(group, rows) {
  const benchmarkCount = new Set(rows.map((row) => row.benchmark)).size;
  const includeBenchmark = benchmarkCount > 1;
  const lines = [
    `##### ${groupTitle(group)}`,
    "",
    "| Benchmark | Merge instructions | Base instructions | Change |",
    "| --- | ---: | ---: | ---: |",
  ];
  for (const row of rows) {
    const label = escapeTableCell(benchmarkLabel(row, includeBenchmark));
    const mergeInstructions = formatInstructionCount(row.mergeInstructions);
    const baseInstructions = formatInstructionCount(row.baseInstructions);
    const change = formatChange(row);
    lines.push(
      `| \`${label}\` | ${mergeInstructions} | ${baseInstructions} | ${change} |`,
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
    throw new Error("No instruction comparisons to summarize");
  }

  const groups = groupRows(rows);
  const tables = Array.from(groups, ([group, rowsInGroup]) =>
    renderGroupTable(group, rowsInGroup),
  ).join("\n");

  return [
    "### Instruction-count changes",
    "",
    "Exact Callgrind instruction counts for the merge and base revisions. Multi-part profiles use the part numbers from Gungraun and include an aggregated Total. New and Removed identify benchmarks that exist on only one side; the missing count is shown as a dash. Percentages use the same precision as the Gungraun text report. Markers: 🟢 fewer instructions, 🔴 more instructions, ⚪ unchanged.",
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
    "### Instruction-count changes",
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
