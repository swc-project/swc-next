import { createWalker, merge } from "@swc-next/walker";
import { encode } from "./ast/encode.js";
import { decode } from "./ast/decode.js";
import { encodeSource } from "./source.js";
import { encodeTransformApiOptions } from "./transform.js";
import { decodeJsOutput } from "./output.js";

const AFTER_PARSE = 1;
const BEFORE_CODEGEN = 2;

export function runTransformPipeline(source, options, bindings) {
  const plugins = compilePlugins(options.plugins);
  const packed = encodeTransformApiOptions(options, true);
  const stages = (plugins.afterParse === null ? 0 : AFTER_PARSE)
    | (plugins.beforeCodegen === null ? 0 : BEFORE_CODEGEN);
  if (stages === 0) {
    return decodeJsOutput(bindings.transformSync(encodeSource(source), source.length, packed));
  }
  const filename = options.filename ?? "input.js";
  const sourceBytes = encodeSource(source).slice();
  let pluginFailure;
  const runJsPlugins = (stage, buffer) => {
    try {
      const result = decode(buffer, source, stage === BEFORE_CODEGEN ? sourceBytes : undefined);
      const program = result.program;
      const walker = stage === AFTER_PARSE ? plugins.afterParse : plugins.beforeCodegen;
      walker.walk(program);
      const parts = encode(program, sourceBytes);
      const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
      const langBits = view.getUint32(32, true) & 28;
      const header = new DataView(parts[0].buffer, parts[0].byteOffset, parts[0].byteLength);
      header.setUint32(32, (header.getUint32(32, true) & ~28) | langBits, true);
      return joinAstParts(parts);
    } catch (error) {
      pluginFailure = { error };
      throw error;
    }
  };
  let result;
  try {
    result = bindings.__transformV1WithHooks(sourceBytes, source.length, packed, stages, runJsPlugins);
  } catch (error) {
    if (pluginFailure !== undefined) throw pluginFailure.error;
    throw new TypeError(error.message);
  }
  const output = decodeJsOutput(result);
  throwOnDiagnostics(output.diagnostics, filename, source);
  return output;
}

function joinAstParts([header, spans, kinds, payloads, extra, strings]) {
  const kindsEnd = header.length + spans.length + kinds.length;
  const payloadStart = Math.ceil(kindsEnd / 8) * 8;
  const length = payloadStart + payloads.length + extra.length + strings.length;
  const bytes = new Uint8Array(Math.ceil(length / 4) * 4);
  let offset = 0;
  for (const part of [header, spans, kinds]) {
    bytes.set(part, offset);
    offset += part.length;
  }
  offset = payloadStart;
  for (const part of [payloads, extra, strings]) {
    bytes.set(part, offset);
    offset += part.length;
  }
  return bytes;
}

function compilePlugins(plugins) {
  const stages = { afterParse: [], beforeCodegen: [] };
  if (plugins === undefined) return { afterParse: null, beforeCodegen: null };
  if (!Array.isArray(plugins)) throw new TypeError("plugins must be an array");
  for (const [index, plugin] of plugins.entries()) {
    const path = `plugins[${index}]`;
    validatePluginObject(plugin, path);
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
    stages[stage].push(plugin.visitor);
  }
  return {
    afterParse: stages.afterParse.length === 0 ? null : compileStage(stages.afterParse),
    beforeCodegen: stages.beforeCodegen.length === 0 ? null : compileStage(stages.beforeCodegen),
  };
}

function compileStage(visitors) {
  return createWalker(visitors.length === 1 ? visitors[0] : merge(visitors));
}

function validatePluginObject(value, path) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${path} must be a plain object`);
  }
  const prototype = Object.getPrototypeOf(value);
  if (prototype !== null && Object.getPrototypeOf(prototype) !== null) {
    throw new TypeError(`${path} must be a plain object`);
  }
  for (const key of Reflect.ownKeys(value)) {
    if (key !== "name" && key !== "stage" && key !== "visitor") {
      throw new TypeError(`Unsupported option ${path}.${String(key)}`);
    }
  }
}

function throwOnDiagnostics(diagnostics, filename, source) {
  const diagnostic = diagnostics.find(diagnostic => diagnostic.severity === "error");
  if (diagnostic === undefined) return;
  const lines = source.slice(0, diagnostic.start).split(/\r\n|[\n\r\u2028\u2029]/);
  const error = new SyntaxError(`${filename}:${lines.length}:${lines.at(-1).length + 1}: ${diagnostic.message}`);
  error.diagnostics = diagnostics;
  throw error;
}
