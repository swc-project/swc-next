import {
  langFromPath,
  parseSync as __parseSync,
  printSync as __printSync,
  sourceTypeFromPath,
  transformAstSync as __transformAstSync,
  transformSync as __transformSync,
} from "./binding.js";
import { createWalker } from "@swc-next/walker";
import {
  decode,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
const emptySourceBytes = new Uint8Array();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";

export function parseSync(source, options) {
  const sourceBytes = encodeSource(source);
  const buffer = __parseSync(sourceBytes, source.length, encodeParserOptions(options));
  const result = decode(buffer, source, sourceBytes);
  if (!options?.errorRecovery) {
    const diagnostics = result.diagnostics;
    if (diagnostics.length) throw diagnostics[0];
  }
  return result;
}

function encodeSource(source) {
  // Every byte read below is initialized by encodeInto; avoid zeroing the
  // oversized scratch buffer on Node. Keep the portable allocation for browsers.
  const sourceBuffer = typeof source === "string" && typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(source.length * 3).buffer)
    : new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Lazy decoder getters retain sourceBytes, so drop the unused capacity of
  // the encodeInto buffer instead of keeping it alive through a subarray.
  return canTransferToFixedLength
    ? new Uint8Array(sourceBuffer.buffer.transferToFixedLength(written))
    : sourceBuffer.slice(0, written);
}

export function printSync(program, options, meta) {
  const code = __printSync(...encode(program, meta), encodeCodegenOptions(options));
  return { code };
}

export function transformSync(source, options) {
  if (typeof source !== "string") throw new TypeError("source must be a string");
  checkOptions(options, "options");
  const filename = options?.filename ?? "input.js";
  if (typeof filename !== "string") throw new TypeError("filename must be a string");
  checkOptions(options?.parser, "parser");
  const parserOptions = {
    ...options?.parser,
    lang: options?.parser?.lang ?? langFromPath(filename),
    sourceType: options?.parser?.sourceType
      ?? (options?.filename == null ? "module" : sourceTypeFromPath(filename)),
    semanticErrors: true,
  };
  const transformOptions = encodeTransformOptions(options?.transform);
  const codegenOptions = encodeCodegenOptions(options?.codegen);
  const plugins = compilePlugins(options?.jsPlugins);

  if (plugins.before.length === 0 && plugins.after.length === 0) {
    const result = decodeTransformOutput(__transformSync(
      encodeSource(source),
      source.length,
      encodeParserOptions(parserOptions),
      transformOptions,
      codegenOptions,
    ));
    throwOnDiagnostics(result.diagnostics, filename);
    return result;
  }

  const parsed = parseSync(source, parserOptions);
  throwOnDiagnostics(parsed.diagnostics, filename);
  const program = parsed.program;
  for (const plugin of plugins.before) plugin.walk(program);

  // The Rust transform owns its AST and returns all strings in the extra pool.
  // Spans remain UTF-16 offsets in the input, independent of the string pool.
  const transformed = decode(
    __transformAstSync(encode(program, parsed.meta), transformOptions),
    "",
    emptySourceBytes,
  );
  const diagnostics = [...parsed.diagnostics, ...transformed.diagnostics];
  throwOnDiagnostics(diagnostics, filename);
  const transformedProgram = transformed.program;
  for (const plugin of plugins.after) plugin.walk(transformedProgram);

  const code = __printSync(
    encode(transformedProgram, transformed.meta),
    codegenOptions,
  );
  return { code, diagnostics };
}

function decodeTransformOutput(buffer) {
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  let offset = 0;
  const readU32 = () => {
    const value = view.getUint32(offset, true);
    offset += 4;
    return value;
  };
  const readText = (length) => {
    const value = textDecoder.decode(buffer.subarray(offset, offset + length));
    offset += length;
    return value;
  };
  const readString = () => readText(readU32());
  const codeLength = readU32();
  const diagnosticCount = readU32();
  const code = readText(codeLength);
  const severities = ["error", "warning", "hint", "info"];
  const diagnostics = [];
  for (let index = 0; index < diagnosticCount; index++) {
    const severity = severities[view.getUint8(offset++)];
    if (severity === undefined) throw new RangeError("Unknown diagnostic severity");
    const start = readU32();
    const end = readU32();
    const message = readString();
    const help = view.getUint8(offset++) ? readString() : undefined;
    const labels = [];
    const labelCount = readU32();
    for (let labelIndex = 0; labelIndex < labelCount; labelIndex++) {
      const start = readU32();
      const end = readU32();
      const message = readString();
      labels.push({ message, start, end });
    }
    diagnostics.push({ severity, message, start, end, help, labels });
  }
  return { code, diagnostics };
}

function compilePlugins(plugins) {
  const phases = { before: [], after: [] };
  if (plugins === undefined) return phases;
  if (!Array.isArray(plugins)) throw new TypeError("jsPlugins must be an array");
  for (const plugin of plugins) {
    if (plugin === null || typeof plugin !== "object" || Array.isArray(plugin)) {
      throw new TypeError("Each JS plugin must be an object");
    }
    if (typeof plugin.name !== "string" || plugin.name.length === 0) {
      throw new TypeError("Each JS plugin must have a non-empty name");
    }
    const phase = plugin.phase ?? "before";
    if (phase !== "before" && phase !== "after") {
      throw new TypeError(`Invalid phase for JS plugin ${plugin.name}`);
    }
    if (
      plugin.visitors === null
      || typeof plugin.visitors !== "object"
      || Array.isArray(plugin.visitors)
    ) {
      throw new TypeError(`JS plugin ${plugin.name} must provide visitors`);
    }
    phases[phase].push(createWalker(plugin.visitors));
  }
  return phases;
}

function encodeTransformOptions(options) {
  checkOptions(options, "transform");
  if (options == null) return 0;
  return 1
    | (encodeBoolean(options.typescript, "typescript") << 1)
    | (encodeBoolean(options.staticBlocks, "staticBlocks") << 3);
}

function encodeBoolean(value, name) {
  if (value == null) return 0;
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
  return value ? 2 : 1;
}

function checkOptions(options, name) {
  if (options != null && (typeof options !== "object" || Array.isArray(options))) {
    throw new TypeError(`${name} must be an object`);
  }
}

function throwOnDiagnostics(diagnostics, filename) {
  const diagnostic = diagnostics.find(diagnostic => diagnostic.severity === "error");
  if (diagnostic === undefined) return;
  const error = new SyntaxError(`${filename}:${diagnostic.start}: ${diagnostic.message}`);
  error.diagnostics = diagnostics;
  throw error;
}
