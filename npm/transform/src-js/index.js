import {
  langFromPath,
  parseTransformSync as __parseTransformSync,
  parseSync as __parseSync,
  printSync as __printSync,
  transformAstSync as __transformAstSync,
  transformPrintSync as __transformPrintSync,
  transformSync as __transformSync,
  validateTransformOptions as __validateTransformOptions,
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

function parseSync(source, options) {
  const sourceBytes = encodeSource(source);
  const buffer = __parseSync(sourceBytes, source.length, encodeParserOptions(options));
  return decode(buffer, source, sourceBytes);
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

export function transformSync(source, options) {
  if (typeof source !== "string") throw new TypeError("source must be a string");
  const { filename, parserOptions, transformOptions, codegenOptions } = normalizeTransformOptions(options);
  const plugins = compilePlugins(options?.plugins);

  if (plugins.before.length === 0 && plugins.after.length === 0) {
    const result = decodeTransformOutput(callTransform(__transformSync,
      encodeSource(source),
      source.length,
      encodeParserOptions(parserOptions),
      transformOptions,
      codegenOptions,
    ));
    throwOnDiagnostics(result.diagnostics, filename);
    return result;
  }

  let transformed;
  let diagnostics;
  if (plugins.before.length === 0) {
    const sourceBytes = encodeSource(source);
    transformed = decode(
      callTransform(__parseTransformSync,
        sourceBytes,
        source.length,
        encodeParserOptions(parserOptions),
        transformOptions,
      ),
      source,
      sourceBytes,
    );
    diagnostics = transformed.diagnostics;
  } else {
    callTransform(__validateTransformOptions, transformOptions);
    const parsed = parseSync(source, parserOptions);
    throwOnDiagnostics(parsed.diagnostics, filename);
    const program = parsed.program;
    for (const plugin of plugins.before) plugin.walk(program);
    const parts = encode(program, parsed.meta);

    if (plugins.after.length === 0) {
      const result = decodeTransformOutput(
        callTransform(__transformPrintSync, ...parts, transformOptions, codegenOptions),
      );
      const diagnostics = [...parsed.diagnostics, ...result.diagnostics];
      throwOnDiagnostics(diagnostics, filename);
      return { code: result.code, diagnostics };
    }

    // A transferred AST has an empty source and strings in the extra pool.
    // Its spans are already UTF-16 offsets in the original input.
    transformed = decode(
      callTransform(__transformAstSync, ...parts, transformOptions),
      "",
      emptySourceBytes,
    );
    diagnostics = [...parsed.diagnostics, ...transformed.diagnostics];
  }
  throwOnDiagnostics(diagnostics, filename);
  const transformedProgram = transformed.program;
  for (const plugin of plugins.after) plugin.walk(transformedProgram);

  const code = __printSync(
    ...encode(transformedProgram, transformed.meta),
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
  if (!Array.isArray(plugins)) throw new TypeError("plugins must be an array");
  for (const [index, plugin] of plugins.entries()) {
    const path = `plugins[${index}]`;
    optionObject(plugin, path, ["name", "phase", "visitors"]);
    if (plugin === undefined) throw new TypeError(`${path} must be an object`);
    if (typeof plugin.name !== "string" || plugin.name.length === 0) {
      throw new TypeError(`${path}.name must be a non-empty string`);
    }
    const phase = plugin.phase === undefined ? "before" : plugin.phase;
    if (phase !== "before" && phase !== "after") {
      throw new TypeError(`${path}.phase must be "before" or "after"`);
    }
    if (
      plugin.visitors === null
      || typeof plugin.visitors !== "object"
      || Array.isArray(plugin.visitors)
    ) {
      throw new TypeError(`${path}.visitors must be an object`);
    }
    phases[phase].push(createWalker(plugin.visitors));
  }
  return phases;
}

function normalizeTransformOptions(options) {
  const config = optionObject(options ?? {}, "", [
    "filename", "parser", "typescript", "es2022", "proposal", "codegen", "plugins",
  ]);
  const filename = config.filename === undefined ? "input.js" : config.filename;
  if (typeof filename !== "string") throw new TypeError("filename must be a string");
  const parser = optionObject(config.parser, "parser", [
    "lang", "sourceType", "preserveParens", "comments", "semanticErrors", "errorRecovery",
  ]);
  if (parser.comments !== undefined && parser.comments !== "none") {
    throw new TypeError('Unsupported option parser.comments: transform only supports "none"');
  }
  if (parser.semanticErrors !== undefined && parser.semanticErrors !== true) {
    throw new TypeError("Unsupported option parser.semanticErrors: transform requires semantic checks");
  }
  if (parser.errorRecovery !== undefined && parser.errorRecovery !== true) {
    throw new TypeError("Unsupported option parser.errorRecovery: transform collects diagnostics before throwing");
  }
  const parserOptions = {
    ...parser,
    lang: parser.lang ?? langFromPath(filename),
    comments: "none",
    semanticErrors: true,
    errorRecovery: true,
  };
  const transformOptions = JSON.stringify({
    typescript: config.typescript,
    es2022: config.es2022,
    proposal: config.proposal,
  });
  const codegen = optionObject(config.codegen, "codegen", ["asciiOnly", "minify", "inlineScript"]);
  const codegenOptions = encodeCodegenOptions(codegen);
  return { filename, parserOptions, transformOptions, codegenOptions };
}

function callTransform(binding, ...args) {
  try {
    return binding(...args);
  } catch (error) {
    if (error instanceof Error && error.message.startsWith("Invalid transform options: ")) {
      throw new TypeError(error.message);
    }
    throw error;
  }
}

function optionObject(value, path, keys) {
  if (value === undefined) return {};
  if (
    value === null || typeof value !== "object" || Array.isArray(value)
    || (Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null)
  ) {
    throw new TypeError(`${path || "options"} must be a plain object`);
  }
  for (const key of Reflect.ownKeys(value)) {
    if (!keys.includes(key)) {
      throw new TypeError(`Unsupported option ${path ? path + "." : ""}${String(key)}`);
    }
  }
  return value;
}

function throwOnDiagnostics(diagnostics, filename) {
  const diagnostic = diagnostics.find(diagnostic => diagnostic.severity === "error");
  if (diagnostic === undefined) return;
  const error = new SyntaxError(`${filename}:${diagnostic.start}: ${diagnostic.message}`);
  error.diagnostics = diagnostics;
  throw error;
}
