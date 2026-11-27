// Internal SWC Next transport; these APIs and formats do not follow semver.
// Keep in sync with crates/swc/src/js/transformer/bridge.rs.
import { storage } from "./buffer.js";
import { encodeParserConfig } from "./parser.js";
import { encodeGeneratorConfig } from "./generator.js";

const encoder = new TextEncoder();
const empty = new Uint8Array();
const comments = ["all", "none"];
const jsxRuntimes = ["classic", "automatic"];
const decoratorVersions = ["2023-11", "legacy"];
const moduleTypes = ["preserve", "commonjs", "amd", "umd", "systemjs"];

const transformApiKeys = new Set([
  "filename",
  "sourceMap",
  "comments",
  "parser",
  "transformer",
  "generator",
]);
const parserKeys = new Set([
  "sourceType",
  "lang",
  "preserveParens",
  "comments",
  "semanticErrors",
]);
const generatorKeys = new Set(["asciiOnly", "inlineScript", "minify"]);
const transformerKeys = new Set([
  "filename",
  "target",
  "typescript",
  "jsx",
  "decorators",
  "module",
  "features",
  "assumptions",
]);
const typescriptKeys = new Set([
  "verbatimModuleSyntax",
  "jsxRuntime",
  "jsxPragma",
  "jsxPragmaFrag",
]);
const jsxKeys = new Set([
  "runtime",
  "development",
  "sourceSelf",
  "importSource",
  "pragma",
  "pragmaFrag",
  "throwIfNamespace",
  "pure",
  "displayName",
  "refresh",
]);
const refreshKeys = new Set([
  "refreshReg",
  "refreshSig",
  "emitFullSignatures",
]);
const decoratorKeys = new Set(["version", "metadata"]);
const moduleKeys = new Set(["type"]);
const featureList = [
  "exponentiation-operator",
  "async-to-generator",
  "async-generator-functions",
  "object-rest-spread",
  "optional-catch-binding",
  "export-namespace-from",
  "nullish-coalescing-operator",
  "optional-chaining",
  "logical-assignment-operators",
  "class-static-block",
  "class-properties",
  "private-property-in-object",
  "dotall-regex",
  "sticky-regex",
  "unicode-regex",
  "unicode-sets-regex",
];
const featureKeys = new Set(featureList);
const assumptionKeys = new Set([
  "constantSuper",
  "ignoreFunctionLength",
  "noDocumentAll",
  "objectRestNoSymbols",
  "privateFieldsAsProperties",
  "pureGetters",
  "setPublicClassFields",
  "setSpreadProperties",
]);

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function assertObject(value, name) {
  if (!isPlainObject(value)) throw new TypeError(`${name} must be an object`);
}

function assertKnownKeys(value, allowed, name) {
  for (const key of Object.keys(value)) {
    if (!allowed.has(key)) throw new TypeError(`Unknown ${name} option: ${key}`);
  }
}

function boolean(value, name) {
  if (value == null) return 0;
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
  return value ? 2 : 1;
}

function string(value, name) {
  if (value == null) return null;
  if (typeof value !== "string") throw new TypeError(`${name} must be a string`);
  return value;
}

function enumeration(value, values, name) {
  if (value == null) return 0;
  const index = values.indexOf(value);
  if (index < 0) throw new TypeError(`Invalid ${name}`);
  return index + 1;
}

function boolOrObject(value, name) {
  if (value == null) return 0;
  if (typeof value === "boolean") return value ? 2 : 1;
  if (isPlainObject(value)) return 3;
  throw new TypeError(`${name} must be a boolean or an object`);
}

let buffer = empty;
let view;
let offset = 0;

function ensure(extra) {
  const needed = offset + extra;
  if (buffer.length >= needed) return;
  const next = storage(Math.max(needed, buffer.length * 2, 256));
  if (offset > 0) next.set(buffer.subarray(0, offset));
  buffer = next;
  view = new DataView(buffer.buffer);
}

function writeU8(value) {
  ensure(1);
  buffer[offset++] = value;
}

function writeU16(value) {
  ensure(2);
  view.setUint16(offset, value, true);
  offset += 2;
}

function writeU32(value) {
  ensure(4);
  view.setUint32(offset, value, true);
  offset += 4;
}

function writeString(value) {
  ensure(4 + value.length * 3);
  const { written } = encoder.encodeInto(value, buffer.subarray(offset + 4));
  view.setUint32(offset, written, true);
  offset += 4 + written;
}

function writeTransformerConfig(transformer) {
  assertKnownKeys(transformer, transformerKeys, "transformer");

  const filename = string(transformer.filename, "transformer.filename");

  const target = transformer.target;
  let targetKind = 0;
  let targetEntries = null;
  if (target != null) {
    if (typeof target === "string") {
      targetKind = 1;
    } else if (Array.isArray(target)) {
      for (let i = 0; i < target.length; i++) {
        if (typeof target[i] !== "string") {
          throw new TypeError("transformer.target entries must be strings");
        }
      }
      targetKind = 2;
    } else if (isPlainObject(target)) {
      targetEntries = Object.entries(target);
      for (let i = 0; i < targetEntries.length; i++) {
        if (typeof targetEntries[i][1] !== "string") {
          throw new TypeError("transformer.target engine versions must be strings");
        }
      }
      targetKind = 3;
    } else {
      throw new TypeError("transformer.target must be a string, string[], or object");
    }
  }

  const tsMode = boolOrObject(transformer.typescript, "transformer.typescript");
  const jsxMode = boolOrObject(transformer.jsx, "transformer.jsx");
  const decMode = boolOrObject(transformer.decorators, "transformer.decorators");

  let moduleCode = 0;
  if (transformer.module != null) {
    assertObject(transformer.module, "transformer.module");
    assertKnownKeys(transformer.module, moduleKeys, "transformer.module");
    moduleCode = 1 + enumeration(transformer.module.type, moduleTypes, "transformer.module.type");
  }

  const hasFeatures = transformer.features != null;
  if (hasFeatures) {
    assertObject(transformer.features, "transformer.features");
    assertKnownKeys(transformer.features, featureKeys, "transformer.features");
  }

  const hasAssumptions = transformer.assumptions != null;
  if (hasAssumptions) {
    assertObject(transformer.assumptions, "transformer.assumptions");
    assertKnownKeys(transformer.assumptions, assumptionKeys, "transformer.assumptions");
  }

  const flags =
    (filename != null ? 0x1 : 0) |
    (targetKind << 1) |
    (tsMode << 3) |
    (jsxMode << 5) |
    (decMode << 7) |
    (moduleCode << 9) |
    (hasFeatures ? 0x1000 : 0) |
    (hasAssumptions ? 0x2000 : 0);
  writeU32(flags);

  if (filename != null) writeString(filename);

  if (targetKind === 1) {
    writeString(target);
  } else if (targetKind === 2) {
    writeU32(target.length);
    for (let i = 0; i < target.length; i++) writeString(target[i]);
  } else if (targetKind === 3) {
    writeU32(targetEntries.length);
    for (let i = 0; i < targetEntries.length; i++) {
      writeString(targetEntries[i][0]);
      writeString(targetEntries[i][1]);
    }
  }

  if (tsMode === 3) {
    const ts = transformer.typescript;
    assertKnownKeys(ts, typescriptKeys, "transformer.typescript");
    const jsxPragma = string(ts.jsxPragma, "transformer.typescript.jsxPragma");
    const jsxPragmaFrag = string(ts.jsxPragmaFrag, "transformer.typescript.jsxPragmaFrag");
    writeU8(
      boolean(ts.verbatimModuleSyntax, "transformer.typescript.verbatimModuleSyntax") |
        (enumeration(ts.jsxRuntime, jsxRuntimes, "transformer.typescript.jsxRuntime") << 2) |
        (jsxPragma != null ? 0x10 : 0) |
        (jsxPragmaFrag != null ? 0x20 : 0),
    );
    if (jsxPragma != null) writeString(jsxPragma);
    if (jsxPragmaFrag != null) writeString(jsxPragmaFrag);
  }

  if (jsxMode === 3) {
    const jsx = transformer.jsx;
    assertKnownKeys(jsx, jsxKeys, "transformer.jsx");
    const importSource = string(jsx.importSource, "transformer.jsx.importSource");
    const pragma = string(jsx.pragma, "transformer.jsx.pragma");
    const pragmaFrag = string(jsx.pragmaFrag, "transformer.jsx.pragmaFrag");
    const refreshMode = boolOrObject(jsx.refresh, "transformer.jsx.refresh");
    writeU32(
      enumeration(jsx.runtime, jsxRuntimes, "transformer.jsx.runtime") |
        (boolean(jsx.development, "transformer.jsx.development") << 2) |
        (boolean(jsx.sourceSelf, "transformer.jsx.sourceSelf") << 4) |
        (boolean(jsx.throwIfNamespace, "transformer.jsx.throwIfNamespace") << 6) |
        (boolean(jsx.pure, "transformer.jsx.pure") << 8) |
        (boolean(jsx.displayName, "transformer.jsx.displayName") << 10) |
        (refreshMode << 12) |
        (importSource != null ? 0x4000 : 0) |
        (pragma != null ? 0x8000 : 0) |
        (pragmaFrag != null ? 0x10000 : 0),
    );
    if (importSource != null) writeString(importSource);
    if (pragma != null) writeString(pragma);
    if (pragmaFrag != null) writeString(pragmaFrag);
    if (refreshMode === 3) {
      const refresh = jsx.refresh;
      assertKnownKeys(refresh, refreshKeys, "transformer.jsx.refresh");
      const refreshReg = string(refresh.refreshReg, "transformer.jsx.refresh.refreshReg");
      const refreshSig = string(refresh.refreshSig, "transformer.jsx.refresh.refreshSig");
      writeU8(
        boolean(refresh.emitFullSignatures, "transformer.jsx.refresh.emitFullSignatures") |
          (refreshReg != null ? 0x04 : 0) |
          (refreshSig != null ? 0x08 : 0),
      );
      if (refreshReg != null) writeString(refreshReg);
      if (refreshSig != null) writeString(refreshSig);
    }
  }

  if (decMode === 3) {
    const dec = transformer.decorators;
    assertKnownKeys(dec, decoratorKeys, "transformer.decorators");
    writeU8(
      enumeration(dec.version, decoratorVersions, "transformer.decorators.version") |
        (boolean(dec.metadata, "transformer.decorators.metadata") << 2),
    );
  }

  if (hasFeatures) {
    const features = transformer.features;
    let word = 0;
    for (let i = 0; i < featureList.length; i++) {
      const key = featureList[i];
      word |= boolean(features[key], `transformer.features["${key}"]`) << (i * 2);
    }
    writeU32(word >>> 0);
  }

  if (hasAssumptions) {
    const a = transformer.assumptions;
    writeU16(
      boolean(a.constantSuper, "transformer.assumptions.constantSuper") |
        (boolean(a.ignoreFunctionLength, "transformer.assumptions.ignoreFunctionLength") << 2) |
        (boolean(a.noDocumentAll, "transformer.assumptions.noDocumentAll") << 4) |
        (boolean(a.objectRestNoSymbols, "transformer.assumptions.objectRestNoSymbols") << 6) |
        (boolean(a.privateFieldsAsProperties, "transformer.assumptions.privateFieldsAsProperties") <<
          8) |
        (boolean(a.pureGetters, "transformer.assumptions.pureGetters") << 10) |
        (boolean(a.setPublicClassFields, "transformer.assumptions.setPublicClassFields") << 12) |
        (boolean(a.setSpreadProperties, "transformer.assumptions.setSpreadProperties") << 14),
    );
  }
}

/** Encode the complete JS transform configuration into reusable scratch storage.
 * The returned buffer is valid until the next call. Option getters and Proxy traps
 * must not reenter this encoder, including via any transformSync entry point.
 */
export function encodeTransformApiOptions(options) {
  if (options == null) return empty;
  assertObject(options, "options");
  assertKnownKeys(options, transformApiKeys, "transform");
  const parser = options.parser;
  if (parser != null) {
    assertObject(parser, "parser");
    assertKnownKeys(parser, parserKeys, "parser");
  }
  const generator = options.generator;
  if (generator != null) {
    assertObject(generator, "generator");
    assertKnownKeys(generator, generatorKeys, "generator");
  }
  const filename = string(options.filename, "filename");
  const transformer = options.transformer;
  if (transformer != null) assertObject(transformer, "transformer");

  offset = 0;
  ensure(16);
  const flags =
    boolean(options.sourceMap, "sourceMap") |
    (enumeration(options.comments, comments, "comments") << 2) |
    (filename != null ? 0x10 : 0) |
    (transformer != null ? 0x20 : 0);
  view.setUint32(0, 1, true);
  view.setUint32(4, encodeParserConfig(parser), true);
  view.setUint32(8, encodeGeneratorConfig(generator), true);
  view.setUint32(12, flags, true);
  offset = 16;

  if (filename != null) writeString(filename);
  if (transformer != null) writeTransformerConfig(transformer);
  return buffer.subarray(0, offset);
}
