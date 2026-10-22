import { closeSync, openSync, readSync, readdirSync } from "node:fs";

export type ConformanceKind = "parser" | "codegen" | "semantic" | "transform" | "estree" | "codec";
export interface Count {
  passed: number;
  total: number;
}
export interface ConformanceResult {
  kind: ConformanceKind;
  suite: string;
  snapshot: string;
  total: number;
  passed: number;
  failed: number;
  positive?: Count;
  negative?: Count;
  parsed?: Count;
  skipped?: number;
  panicked?: number;
}

const kinds = new Set<ConformanceKind>(["parser", "codegen", "semantic", "transform", "estree", "codec"]);

export function parseConformanceHeader(snapshot: string, header: string): ConformanceResult {
  const name = snapshot.match(/^([a-z]+)_(.+)\.snap$/);
  if (!name || !kinds.has(name[1] as ConformanceKind)) throw new Error(`Unknown conformance snapshot: ${snapshot}`);
  const kind = name[1] as ConformanceKind;
  const suite = name[2];
  const lines = header.split(/\r?\n/);
  const fields = new Map<string, { value: number; total?: number }>();
  let start = 0;
  if (!["estree", "codec"].includes(kind)) {
    if (lines[0] !== `${kind} * ${suite}` || !/^=+$/.test(lines[1])) throw new Error(`Invalid conformance header: ${snapshot}`);
    start = 2;
  }
  for (const line of lines.slice(start)) {
    const match = line.match(/^([A-Za-z ]+?)\s*:\s*(\d+)(?:\/(\d+))?(?:\s+\([\d.]+%\))?$/);
    if (!match) {
      if (/^(Positive Passed|Negative Passed|Panicked|Skipped|AST Parsed|Codec Passed|Cases)\s*:/.test(line)) throw new Error(`Invalid summary field: ${snapshot}`);
      break;
    }
    const label = match[1].trim();
    if (fields.has(label)) throw new Error(`Duplicate ${label}: ${snapshot}`);
    const value = Number(match[2]);
    const total = match[3] === undefined ? undefined : Number(match[3]);
    if (!Number.isSafeInteger(value) || (total !== undefined && (!Number.isSafeInteger(total) || value > total))) throw new Error(`Invalid ${label}: ${snapshot}`);
    fields.set(label, { value, total });
  }
  const count = (label: string): Count => {
    const field = fields.get(label);
    if (!field || field.total === undefined) throw new Error(`Missing ${label}: ${snapshot}`);
    return { passed: field.value, total: field.total };
  };
  if (kind === "codec") {
    const passed = count("Codec Passed");
    if (fields.get("Cases")?.value !== passed.total) throw new Error(`Inconsistent codec counts: ${snapshot}`);
    return { kind, suite, snapshot, total: passed.total, passed: passed.passed, failed: passed.total - passed.passed };
  }
  if (kind === "estree") {
    const parsed = count("AST Parsed");
    const passed = count("Positive Passed");
    if (parsed.total !== passed.total || passed.passed > parsed.passed) throw new Error(`Inconsistent ESTree counts: ${snapshot}`);
    return { kind, suite, snapshot, total: passed.total, passed: passed.passed, failed: passed.total - passed.passed, parsed };
  }
  const positive = count("Positive Passed");
  const negative = count("Negative Passed");
  const panicked = count("Panicked");
  const skipped = fields.has("Skipped") ? count("Skipped") : { passed: 0, total: panicked.total };
  const total = positive.total + negative.total;
  const passed = positive.passed + negative.passed;
  const failed = total - passed - skipped.passed - panicked.passed;
  if (total !== panicked.total || total !== skipped.total || failed < 0 || skipped.passed > positive.total - positive.passed) throw new Error(`Inconsistent native counts: ${snapshot}`);
  return { kind, suite, snapshot, total, passed, failed, positive, negative, skipped: skipped.passed, panicked: panicked.passed };
}

export function loadConformanceReport(directory = new URL("../../tasks/testsuite/snapshots/", import.meta.url)): ConformanceResult[] {
  const files = readdirSync(directory).filter((file) => file.endsWith(".snap")).sort();
  if (!files.length) throw new Error("No conformance snapshots found");
  return files.map((snapshot) => {
    const file = openSync(new URL(snapshot, directory), "r");
    const buffer = Buffer.alloc(4096);
    try {
      const length = readSync(file, buffer, 0, buffer.length, 0);
      return parseConformanceHeader(snapshot, buffer.toString("utf8", 0, length));
    } finally {
      closeSync(file);
    }
  });
}
