import {
  langFromPath,
  parseTransformSync as __parseTransformSync,
  parseSync as __parseSync,
  printSync as __printSync,
  transformAstSync as __transformAstSync,
  transformPrintSync as __transformPrintSync,
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

  let transformed;
  let diagnostics;
  if (plugins.before.length === 0) {
    const sourceBytes = encodeSource(source);
    transformed = decode(
      __parseTransformSync(
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
    const parsed = parseSync(source, parserOptions);
    throwOnDiagnostics(parsed.diagnostics, filename);
    const program = parsed.program;
    for (const plugin of plugins.before) plugin.walk(program);
    const buffer = encode(program, parsed.meta);

    if (plugins.after.length === 0) {
      const result = decodeTransformOutput(
        __transformPrintSync(buffer, transformOptions, codegenOptions),
      );
      const diagnostics = [...parsed.diagnostics, ...result.diagnostics];
      throwOnDiagnostics(diagnostics, filename);
      return { code: result.code, diagnostics };
    }

    // A transferred AST has an empty source and strings in the extra pool.
    // Its spans are already UTF-16 offsets in the original input.
    transformed = decode(
      __transformAstSync(buffer, transformOptions),
      "",
      emptySourceBytes,
    );
    diagnostics = [...parsed.diagnostics, ...transformed.diagnostics];
  }
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
  const config = optionObject(options ?? {}, "", ["filename", "isModule", "jsc", "minify", "plugins", "swcNext"]);
  const filename = config.filename === undefined ? "input.js" : config.filename;
  if (typeof filename !== "string") throw new TypeError("filename must be a string");
  const jsc = optionObject(config.jsc, "jsc", ["parser", "target", "transform", "output"]);
  const transform = optionObject(jsc.transform, "jsc.transform", ["verbatimModuleSyntax"]);
  const output = optionObject(jsc.output, "jsc.output", ["charset"]);
  const extensions = optionObject(config.swcNext, "swcNext", [
    "typescript", "staticBlocks", "preserveParens", "inlineScript", "preserveConstEnums",
  ]);

  if (jsc.target !== undefined && jsc.target !== "esnext") {
    throw new TypeError('Unsupported option jsc.target: only "esnext" is supported');
  }
  if (output.charset !== undefined && output.charset !== "ascii" && output.charset !== "utf8") {
    throw new TypeError('jsc.output.charset must be "ascii" or "utf8"');
  }
  let sourceType;
  switch (config.isModule) {
    case undefined:
    case true: sourceType = "module"; break;
    case false: sourceType = "script"; break;
    case "commonjs": sourceType = "commonjs"; break;
    default: throw new TypeError('Unsupported option isModule: expected true, false, or "commonjs"');
  }

  let lang = langFromPath(filename);
  if (jsc.parser !== undefined) {
    const parser = optionObject(jsc.parser, "jsc.parser", ["syntax", "jsx", "tsx"]);
    if (parser.syntax === "ecmascript") {
      optionObject(parser, "jsc.parser", ["syntax", "jsx"]);
      lang = booleanOption(parser.jsx, "jsc.parser.jsx") ? "jsx" : "js";
    } else if (parser.syntax === "typescript") {
      optionObject(parser, "jsc.parser", ["syntax", "tsx"]);
      const tsx = booleanOption(parser.tsx, "jsc.parser.tsx");
      lang = lang === "dts" ? "dts" : tsx ? "tsx" : "ts";
    } else {
      throw new TypeError('jsc.parser.syntax must be "ecmascript" or "typescript"');
    }
  }

  const parserOptions = {
    lang,
    sourceType,
    preserveParens: booleanOption(extensions.preserveParens, "swcNext.preserveParens"),
    comments: "none",
    semanticErrors: true,
  };
  const transformOptions = 1
    | (encodeBoolean(extensions.typescript, "swcNext.typescript") << 1)
    | (encodeBoolean(extensions.staticBlocks, "swcNext.staticBlocks") << 3)
    | (encodeBoolean(transform.verbatimModuleSyntax, "jsc.transform.verbatimModuleSyntax") << 5)
    | (encodeBoolean(extensions.preserveConstEnums, "swcNext.preserveConstEnums") << 7);
  const codegenOptions = encodeCodegenOptions({
    asciiOnly: output.charset === "ascii",
    minify: booleanOption(config.minify, "minify"),
    inlineScript: booleanOption(extensions.inlineScript, "swcNext.inlineScript"),
  });
  return { filename, parserOptions, transformOptions, codegenOptions };
}

function encodeBoolean(value, path) {
  const boolean = booleanOption(value, path);
  return boolean === undefined ? 0 : boolean ? 2 : 1;
}

function booleanOption(value, path) {
  if (value !== undefined && typeof value !== "boolean") {
    throw new TypeError(`${path} must be a boolean`);
  }
  return value;
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
