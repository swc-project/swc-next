// Reproduce the formatter's private display-width data from the pinned Prettier
// checkout and its already installed modules. No install or network is needed.
// node crates/swc_ecma_formatter/scripts/generate-width-data.mjs <prettier> [--check] [--evidence <directory>]
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const prettier = path.resolve(process.argv[2]);
const check = process.argv.includes("--check");
const evidenceIndex = process.argv.indexOf("--evidence");
const evidence = evidenceIndex === -1 ? null : path.resolve(process.argv[evidenceIndex + 1]);
const sha = "9b47fe6c74994edf85fb4660e3c66b364c784eb2";
const pins = {
  "src/utilities/get-string-width.js": "501f7bb907e13da496a78c983d0b8d7f58fe89b4284a503fece6f04c4d226258",
  "node_modules/emoji-regex/index.mjs": "557dcc678161aac912a3a1db1a464b6062f706303b4622689f2d6a1350fb049a",
  "node_modules/emoji-regex/package.json": "46c5e6b18c74cf3b188d9d49ed79b7df32594deb4e250c5e7f41f87cd6266639",
  "node_modules/get-east-asian-width/index.js": "738e318572295ca05aefac2a13e00a84837fa558fc7bd26ffbf49ecc698339a2",
  "node_modules/get-east-asian-width/lookup.js": "6471cd0b3ecfadd326ff8597e54bb0bac491f011d730bc52943f59f539296d36",
  "node_modules/get-east-asian-width/lookup-data.js": "46be0cad7090de371217389a1c427737a76c92190467a6af82f4adfa5196cee7",
  "node_modules/get-east-asian-width/utilities.js": "4b08a7e9e3ffacbcf198a6abceb2338d52ac671899e52ccc2851c898bfccac42",
  "node_modules/get-east-asian-width/package.json": "25b97bb720e84cdb3b0c3d1c1e804ce6b216d2573ff79cbe531e01bfa05ee9e9",
  "node_modules/narrow-emojis/index.js": "5a5c50bd94927dac0ca7bb06c33e0ecd077737dbccb51485ce4ad2b3519876b9",
  "node_modules/narrow-emojis/package.json": "372fab3d26bc95f644ac07cf704d99f747e56f87e0c1e1273398d10ceb5d8080",
};
const hash = (data) => crypto.createHash("sha256").update(data).digest("hex");
assert.equal(execFileSync("git", ["rev-parse", "HEAD"], { cwd: prettier, encoding: "utf8" }).trim(), sha);
assert.equal(execFileSync("git", ["status", "--porcelain"], { cwd: prettier, encoding: "utf8" }), "");
assert.equal(process.versions.unicode, "17.0");
for (const [relative, expected] of Object.entries(pins)) {
  assert.equal(hash(fs.readFileSync(path.join(prettier, relative))), expected, relative);
}
const load = (relative) => import(pathToFileURL(path.join(prettier, relative)));
const { default: emojiRegex } = await load("node_modules/emoji-regex/index.mjs");
const { isNarrowEmojiCharacter } = await load("node_modules/narrow-emojis/index.js");
const { _isWide: isWide, _isFullwidth: isFullwidth } = await load("node_modules/get-east-asian-width/index.js");
const { default: getStringWidth } = await load("src/utilities/get-string-width.js");
const expression = emojiRegex().source;

// This pinned expression is a finite language: classes, concatenation, ordered
// alternatives and greedy optionals. Reject other grammar rather than silently
// approximating a new upstream expression or its leftmost-first semantics.
let position = 0;
function unit() {
  const character = expression[position++];
  if (character !== "\\") return character.charCodeAt(0);
  const kind = expression[position++];
  const size = { u: 4, x: 2 }[kind];
  assert.ok(size, "unsupported emoji escape");
  const value = Number.parseInt(expression.slice(position, position + size), 16);
  assert.ok(Number.isFinite(value));
  position += size;
  return value;
}
function atom() {
  let sequences;
  if (expression.startsWith("(?:", position)) {
    position += 3;
    sequences = alternatives();
    assert.equal(expression[position++], ")");
  } else if (expression[position] === "[") {
    position++;
    const units = [];
    while (expression[position] !== "]") {
      assert.ok(position < expression.length, "unterminated emoji class");
      const first = unit();
      if (expression[position] === "-" && expression[position + 1] !== "]") {
        position++;
        const last = unit();
        assert.ok(first <= last);
        for (let value = first; value <= last; value++) units.push(value);
      } else {
        units.push(first);
      }
    }
    position++;
    sequences = units.map((value) => [value]);
  } else {
    assert.ok(!"*+{}^$.[".includes(expression[position]), "unsupported emoji grammar");
    sequences = [[unit()]];
  }
  if (expression[position] === "?") {
    position++;
    sequences.push([]); // Greedy optional: the present branch has priority.
  }
  return sequences;
}
function concatenation() {
  let sequences = [[]];
  while (position < expression.length && !")|".includes(expression[position])) {
    const next = atom();
    sequences = sequences.flatMap((prefix) => next.map((suffix) => [...prefix, ...suffix]));
  }
  return sequences;
}
function alternatives() {
  let sequences = concatenation();
  while (expression[position] === "|") {
    position++;
    sequences.push(...concatenation());
  }
  return sequences;
}
const paths = alternatives().map((units) => String.fromCharCode(...units));
assert.equal(position, expression.length);
assert.equal(new Set(paths).size, paths.length, "emoji paths retain unique ordered acceptance");
const anchored = new RegExp(`^(?:${expression})`);
const trie = () => ({ edges: new Map(), width: 0, ordinal: null });
const root = trie();
let trieNodes = 1;
for (const [ordinal, sequence] of paths.entries()) {
  assert.equal(anchored.exec(sequence)?.[0], sequence, "enumerated expression path");
  let state = root;
  for (const character of sequence) {
    const point = character.codePointAt(0);
    assert.ok(point < 0xd800 || point > 0xdfff, "emoji paths contain complete Unicode scalars");
    let next = state.edges.get(point);
    if (!next) {
      next = trie();
      state.edges.set(point, next);
      trieNodes++;
    }
    state = next;
  }
  state.width = isNarrowEmojiCharacter(sequence) ? 1 : 2;
  state.ordinal = ordinal;
}
// A longest accepted prefix is valid for this expression only if every longer
// descendant has earlier alternative priority than its accepted ancestor.
let prefixPairs = 0;
function priorities(state) {
  const ordinals = [...state.edges.values()].flatMap(priorities);
  if (state.ordinal !== null) {
    for (const ordinal of ordinals) {
      assert.ok(ordinal < state.ordinal, "emoji expression no longer permits longest-prefix matching");
      prefixPairs++;
    }
    ordinals.push(state.ordinal);
  }
  return ordinals;
}
priorities(root);
const states = [];
const shapes = new Map();
function intern(state) {
  const edges = [...state.edges].sort(([a], [b]) => a - b).map(([point, next]) => [point, intern(next)]);
  const signature = JSON.stringify([state.width, edges]);
  const found = shapes.get(signature);
  if (found !== undefined) return found;
  const index = states.length;
  shapes.set(signature, index);
  const ranges = [];
  for (const [point, next] of edges) {
    const last = ranges.at(-1);
    if (last && last[1] + 1 === point && last[2] === next) last[1] = point;
    else ranges.push([point, point, next]);
  }
  states.push({ width: state.width, edges: ranges });
  return index;
}
const start = intern(root);
const edgeCount = states.reduce((total, state) => total + state.edges.length, 0);
assert.ok(states.length <= 0xffff && edgeCount <= 0xffff, "emoji table offsets fit u16");
function match(sequence) {
  let state = states[start];
  let accepted = null;
  let units = 0;
  for (const character of sequence) {
    const point = character.codePointAt(0);
    const edge = state.edges.find(([first, last]) => first <= point && point <= last);
    if (!edge) break;
    state = states[edge[2]];
    units += character.length;
    if (state.width) accepted = { text: sequence.slice(0, units), width: state.width };
  }
  return accepted;
}
const checks = [];
for (const sequence of paths) {
  for (const suffix of ["", "a", "\u200d", "中"]) {
    const source = sequence + suffix;
    const actual = anchored.exec(source)?.[0] ?? null;
    const table = match(source);
    assert.equal(table?.text ?? null, actual);
    checks.push({ source, actual, table, width: getStringWidth(source) });
  }
}
const prefixes = new Set();
for (const sequence of paths) {
  const scalars = [...sequence];
  for (let length = 0; length < scalars.length; length++) prefixes.add(scalars.slice(0, length).join(""));
}
for (const prefix of prefixes) {
  const source = prefix + "中";
  const actual = anchored.exec(source)?.[0] ?? null;
  const table = match(source);
  assert.equal(table?.text ?? null, actual);
  checks.push({ source, actual, table, width: getStringWidth(source) });
}
function scalarRanges(predicate) {
  const ranges = [];
  let first = null;
  for (let point = 0; point <= 0x110000; point++) {
    const present = point < 0x110000 && predicate(point);
    if (present && first === null) first = point;
    if (!present && first !== null) {
      ranges.push([first, point - 1]);
      first = null;
    }
  }
  return ranges;
}
const marks = /[\p{Nonspacing_Mark}\p{Enclosing_Mark}]/u;
const zero = scalarRanges((point) => marks.test(String.fromCodePoint(point)));
const wide = scalarRanges((point) => isWide(point) || isFullwidth(point));
const hex = (point) => `0x${point.toString(16).toUpperCase()}`;
let output = `// Generated by scripts/generate-width-data.mjs; do not edit.\n// Prettier ${sha}, Node Unicode 17.0 (Mn/Me),\n// get-east-asian-width 1.7.0 (W/F), emoji-regex 11.0.0, narrow-emojis 0.0.3.\n// Upstream MIT notices are retained in DATA-LICENSE.\n\nuse super::{EmojiEdge, EmojiState};\n\npub(super) const EMOJI_START: usize = ${start};\n`;
function ranges(name, values) {
  return `\n#[rustfmt::skip]\npub(super) static ${name}: [(u32, u32); ${values.length}] = [\n${values.map(([first, last]) => `    (${hex(first)}, ${hex(last)}),`).join("\n")}\n];\n`;
}
output += ranges("ZERO_WIDTH_MARKS", zero);
output += ranges("WIDE", wide);
const flatEdges = [];
const nodeRows = states.map((state) => {
  const first = flatEdges.length;
  flatEdges.push(...state.edges);
  return `    EmojiState { first: ${first}, count: ${state.edges.length}, width: ${state.width} },`;
});
output += `\n#[rustfmt::skip]\npub(super) static EMOJI_STATES: [EmojiState; ${states.length}] = [\n${nodeRows.join("\n")}\n];\n`;
output += `\n#[rustfmt::skip]\npub(super) static EMOJI_EDGES: [EmojiEdge; ${flatEdges.length}] = [\n${flatEdges.map(([first, last, next]) => `    EmojiEdge { first: ${hex(first)}, last: ${hex(last)}, next: ${next} },`).join("\n")}\n];\n`;
const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const destination = path.resolve(scriptDir, "../src/text_width/data.rs");
const licenseDestination = path.resolve(scriptDir, "../src/text_width/DATA-LICENSE");
const license = ["emoji-regex/LICENSE-MIT.txt", "get-east-asian-width/license", "narrow-emojis/license"]
  .map((relative) => `${relative}\n\n${fs.readFileSync(path.join(prettier, "node_modules", relative), "utf8").trim()}\n`).join("\n");
for (const [file, contents] of [[destination, output], [licenseDestination, license]]) {
  if (check) assert.equal(fs.readFileSync(file, "utf8"), contents, file);
  else { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, contents); }
}
const report = { prettier: sha, node: process.version, unicode: process.versions.unicode, pins,
  emojiPaths: paths.length, singleScalar: paths.filter((sequence) => [...sequence].length === 1).length,
  maximumScalars: Math.max(...paths.map((sequence) => [...sequence].length)), trieNodes, states: states.length,
  edges: edgeCount, zeroRanges: zero.length, wideRanges: wide.length, prefixPairs,
  matcherChecks: checks.length, checksSha256: hash(checks.map((record) => JSON.stringify(record) + "\n").join("")),
  generatedSha256: hash(output), licenseSha256: hash(license), check };
if (evidence) {
  fs.mkdirSync(evidence, { recursive: true });
  fs.writeFileSync(path.join(evidence, "matcher-checks.jsonl"), checks.map((record) => JSON.stringify(record) + "\n").join(""));
  fs.writeFileSync(path.join(evidence, "receipt.json"), JSON.stringify(report, null, 2) + "\n");
}
console.log(JSON.stringify(report));
