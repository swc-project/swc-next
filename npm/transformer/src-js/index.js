import {
  parseForTransformSync as __parseForTransformSync,
  parseTransformSync as __parseTransformSync,
  printTransformedSync as __printTransformedSync,
  transformAstSync as __transformAstSync,
  transformPrintSync as __transformPrintSync,
  transformSync as __transformSync,
} from "./binding.js";
import { createWalker } from "@swc-next/walker";
import { decode } from "@swc-next/decoder";
import { encode } from "@swc-next/encoder";

const emptySourceBytes = new Uint8Array();
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

function encodeSource(source) {
  // Every byte read below is initialized by encodeInto; avoid zeroing the
  // oversized scratch buffer on Node. Keep the portable allocation for browsers.
  const sourceBuffer = typeof source === "string" && typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(source.length * 3).buffer)
    : new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Pass only the initialized UTF-8 bytes to the Rust parser.
  return canTransferToFixedLength
    ? new Uint8Array(sourceBuffer.buffer.transferToFixedLength(written))
    : sourceBuffer.slice(0, written);
}

export function transformSync(source, options) {
  if (typeof source !== "string") throw new TypeError("source must be a string");
  if (options != null && (typeof options !== "object" || Array.isArray(options))) {
    throw new TypeError("options must be an object");
  }
  const { plugins: pluginOptions, ...config } = options ?? {};
  const plugins = compilePlugins(pluginOptions);
  const optionsJson = JSON.stringify(config);
  const filename = config.filename ?? "input.js";
  const sourceBytes = encodeSource(source);

  if (plugins.afterParse.length === 0 && plugins.beforeCodegen.length === 0) {
    const result = decodeTransformOutput(callTransform(__transformSync,
      sourceBytes, source.length, optionsJson,
    ));
    throwOnDiagnostics(result.diagnostics, filename);
    return result;
  }

  let transformed;
  let diagnostics;
  if (plugins.afterParse.length === 0) {
    transformed = decode(
      callTransform(__parseTransformSync, sourceBytes, source.length, optionsJson),
      source, sourceBytes, false,
    );
    diagnostics = transformed.diagnostics;
  } else {
    const parsedBuffer = callTransform(__parseForTransformSync, sourceBytes, source.length, optionsJson);
    const parsed = decode(parsedBuffer, source, sourceBytes);
    throwOnDiagnostics(parsed.diagnostics, filename);
    const flags = new DataView(parsedBuffer.buffer, parsedBuffer.byteOffset, parsedBuffer.byteLength).getUint32(32, true);
    const lang = (flags >> 2) & 7;
    const jsx = config.transformer?.jsx;
    const jsxEnabled = jsx == null
      ? lang === 2 || lang === 3
      : jsx !== false;
    // Match the Rust JSX pass's header and tag boundaries, not incidental mentions.
    const headerEnd = parsed.program.body[0]?.start ?? Infinity;
    if (jsxEnabled && parsed.comments.some(comment => comment.end <= headerEnd
      && /(?:^|[ \t\r\n*])@jsx(?:Runtime|ImportSource|Frag)?[ \t]+[^ \t\r\n\f]/.test(comment.value))) {
      throw new TypeError("JSX pragma comments are unsupported with afterParse plugins: AST transfers do not retain comments");
    }
    const program = parsed.program;
    for (const plugin of plugins.afterParse) plugin.walk(program);
    const parts = encode(program);

    if (plugins.beforeCodegen.length === 0) {
      const result = decodeTransformOutput(callTransform(__transformPrintSync, ...parts, optionsJson));
      result.diagnostics = [...parsed.diagnostics, ...result.diagnostics];
      throwOnDiagnostics(result.diagnostics, filename);
      return result;
    }

    // Edited AST transfers have UTF-16 spans and strings in the extra pool.
    transformed = decode(
      callTransform(__transformAstSync, ...parts, optionsJson),
      "", emptySourceBytes, false,
    );
    diagnostics = [...parsed.diagnostics, ...transformed.diagnostics];
  }
  throwOnDiagnostics(diagnostics, filename);
  const program = transformed.program;
  for (const plugin of plugins.beforeCodegen) plugin.walk(program);
  const result = decodeTransformOutput(callTransform(__printTransformedSync,
    ...encode(program), optionsJson,
  ));
  result.diagnostics = [...diagnostics, ...result.diagnostics];
  throwOnDiagnostics(result.diagnostics, filename);
  return result;
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
  const mapLength = readU32();
  const diagnosticCount = readU32();
  const code = readText(codeLength);
  const map = mapLength === 0 ? undefined : readText(mapLength);
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
  return map === undefined ? { code, diagnostics } : { code, map, diagnostics };
}

function compilePlugins(plugins) {
  const stages = { afterParse: [], beforeCodegen: [] };
  if (plugins === undefined) return stages;
  if (!Array.isArray(plugins)) throw new TypeError("plugins must be an array");
  for (const [index, plugin] of plugins.entries()) {
    const path = `plugins[${index}]`;
    optionObject(plugin, path, ["name", "stage", "visitor"]);
    if (plugin === undefined) throw new TypeError(`${path} must be an object`);
    if (typeof plugin.name !== "string" || plugin.name.length === 0) {
      throw new TypeError(`${path}.name must be a non-empty string`);
    }
    const stage = plugin.stage === undefined ? "afterParse" : plugin.stage;
    if (stage !== "afterParse" && stage !== "beforeCodegen") {
      throw new TypeError(`${path}.stage must be "afterParse" or "beforeCodegen"`);
    }
    if (
      plugin.visitor === null
      || typeof plugin.visitor !== "object"
      || Array.isArray(plugin.visitor)
    ) {
      throw new TypeError(`${path}.visitor must be an object`);
    }
    stages[stage].push(createWalker(plugin.visitor));
  }
  return stages;
}

function optionObject(value, path, keys) {
  if (value === undefined) return {};
  if (
    value === null || typeof value !== "object" || Array.isArray(value)
    || (Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null)
  ) {
    throw new TypeError(`${path || "options"} must be a plain object`);
  }
  for (const key of keys === undefined ? [] : Reflect.ownKeys(value)) {
    if (!keys.includes(key)) {
      throw new TypeError(`Unsupported option ${path ? path + "." : ""}${String(key)}`);
    }
  }
  return value;
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

function throwOnDiagnostics(diagnostics, filename) {
  const diagnostic = diagnostics.find(diagnostic => diagnostic.severity === "error");
  if (diagnostic === undefined) return;
  const error = new SyntaxError(`${filename}:${diagnostic.start}: ${diagnostic.message}`);
  error.diagnostics = diagnostics;
  throw error;
}
