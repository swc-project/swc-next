import { createWalker } from "@swc-next/walker";
import { encode } from "./ast/encode.js";
import { decode } from "./ast/decode.js";
import { encodeSource } from "./source.js";
import { encodeTransformApiOptions } from "./transform.js";
import { decodeJsOutput } from "./output.js";

export function transformWithPlugins(source, options, bindings) {
  const { plugins: configured, ...config } = options;
  const plugins = compilePlugins(configured);
  // Validate all current Rust-backed options before running any callbacks.
  let packed = encodeTransformApiOptions(config);
  if (plugins.afterParse.length === 0 && plugins.beforeCodegen.length === 0) {
    return decodeJsOutput(bindings.transformSync(encodeSource(source), source.length, packed));
  }
  // Callbacks may reenter transformSync and overwrite option scratch storage.
  packed = packed.slice();
  if (config.sourceMap) throw new TypeError("sourceMap is unsupported with JavaScript plugins");
  let buffer;
  try { buffer = bindings.parseForPluginsSync(encodeSource(source), source.length, packed); }
  catch (error) { throw new TypeError(error.message); }
  const parsed = decode(buffer, source);
  const filename = config.filename ?? "input.js";
  throwOnDiagnostics(parsed.diagnostics, filename);
  const program = parsed.program;
  const langBits = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength).getUint32(32, true) & 28;
  const encodeProgram = () => {
    const parts = encode(program);
    const header = new DataView(parts[0].buffer, parts[0].byteOffset, parts[0].byteLength);
    // Preserve configured JSX/DTS language; sourceType still comes from Program edits.
    header.setUint32(32, (header.getUint32(32, true) & ~28) | langBits, true);
    return parts;
  };
  for (const plugin of plugins.afterParse) plugin.walk(program);
  let diagnostics = parsed.diagnostics;
  if (plugins.afterParse.length && config.parser?.semanticErrors) {
    const checked = decodeJsOutput(bindings.checkPluginAstSync(...encodeProgram(), packed));
    throwOnDiagnostics(checked.diagnostics, filename);
    diagnostics = [...diagnostics, ...checked.diagnostics];
  }
  for (const plugin of plugins.beforeCodegen) plugin.walk(program);
  const output = decodeJsOutput(bindings.printPluginAstSync(...encodeProgram(), packed));
  output.diagnostics = [...diagnostics, ...output.diagnostics];
  return output;
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

function throwOnDiagnostics(diagnostics, filename) {
  const diagnostic = diagnostics.find(diagnostic => diagnostic.severity === "error");
  if (diagnostic === undefined) return;
  const error = new SyntaxError(`${filename}:${diagnostic.start}: ${diagnostic.message}`);
  error.diagnostics = diagnostics;
  throw error;
}
