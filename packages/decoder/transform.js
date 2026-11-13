// Binary transform transport. Keep the layout aligned with
// crates/swc_core/src/js/transformer.rs.
const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
const empty = new Uint8Array();
const rootKeys = ["filename", "sourceMap", "comments", "parser", "transformer", "generator"];
const parserKeys = ["sourceType", "lang", "preserveParens", "comments", "semanticErrors"];
const generatorKeys = ["asciiOnly", "minify", "inlineScript"];
const transformerKeys = ["filename", "typescript", "jsx", "features", "assumptions", "proposal"];
const typescriptKeys = ["verbatimModuleSyntax", "preserveConstEnums", "jsxRuntime", "jsxPragma", "jsxPragmaFrag"];
const jsxKeys = ["runtime", "development", "sourceSelf", "importSource", "pragma", "pragmaFrag", "throwIfNamespace", "pure", "displayName", "refresh"];
const refreshKeys = ["refreshReg", "refreshSig", "emitFullSignatures"];
const featureKeys = ["class-static-block", "optional-catch-binding"];
const assumptionKeys = [
  "arrayLikeIsIterable", "constantReexports", "constantSuper", "enumerableModuleMeta",
  "ignoreFunctionLength", "ignoreFunctionName", "ignoreToPrimitiveHint", "iterableIsArray",
  "mutableTemplateObject", "noClassCalls", "noDocumentAll", "noIncompleteNsImportDetection",
  "noNewArrows", "objectRestNoSymbols", "privateFieldsAsProperties", "pureGetters",
  "setClassMethods", "setComputedProperties", "setPublicClassFields", "setSpreadProperties",
  "skipForOfIteratorClosing", "superIsCallableConstructor",
];
const languages = ["js", "ts", "jsx", "tsx", "dts"];
const sourceTypes = ["module", "script", "commonjs", "unambiguous"];
const comments = ["all", "none"];
const runtimes = ["classic", "automatic"];

function object(value, keys, name) {
  if (value == null) return;
  if (typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${name} must be an object`);
  for (const key of Object.keys(value)) {
    if (!keys.includes(key)) throw new TypeError(`Unknown ${name} option: ${key}`);
  }
}
function boolean(value, name) {
  if (value == null) return 0;
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
  return value ? 2 : 1;
}
function enumeration(value, values, name) {
  if (value == null) return 0;
  const index = values.indexOf(value);
  if (index < 0) throw new TypeError(`Invalid ${name}`);
  return index + 1;
}
function stage(value, keys, name) {
  if (value == null) return 0;
  if (typeof value === "boolean") return value ? 2 : 1;
  object(value, keys, name);
  return 3;
}
function storage(length) {
  return typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(length).buffer)
    : new Uint8Array(length);
}

/** Create an options encoder with synchronous, reusable scratch storage.
 * The encoder is not reentrant. Option getters and Proxy traps must not
 * reenter it, including via its transformSync wrapper.
 */
export function createTransformOptionsEncoder() {
  let buffer = empty;
  let view;
  const strings = new Array(9);
  return function encodeOptions(options) {
    if (options == null) return empty;
    object(options, rootKeys, "options");
    const parser = options.parser;
    const generator = options.generator;
    const transform = options.transformer;
    object(parser, parserKeys, "parser");
    object(generator, generatorKeys, "generator");
    object(transform, transformerKeys, "transformer");
    const ts = transform?.typescript;
    const jsx = transform?.jsx;
    const tsStage = stage(ts, typescriptKeys, "typescript");
    const jsxStage = stage(jsx, jsxKeys, "jsx");
    const refresh = jsx?.refresh;
    const refreshStage = stage(refresh, refreshKeys, "refresh");
    object(transform?.features, featureKeys, "features");
    object(transform?.assumptions, assumptionKeys, "assumptions");
    object(transform?.proposal, ["preventExtensions"], "proposal");
    strings[0] = options.filename;
    strings[1] = transform?.filename;
    strings[2] = ts?.jsxPragma;
    strings[3] = ts?.jsxPragmaFrag;
    strings[4] = jsx?.importSource;
    strings[5] = jsx?.pragma;
    strings[6] = jsx?.pragmaFrag;
    strings[7] = refresh?.refreshReg;
    strings[8] = refresh?.refreshSig;
    let capacity = 84;
    let hasStrings = false;
    for (const value of strings) {
      if (value == null) continue;
      if (typeof value !== "string") throw new TypeError("transform string options must be strings");
      hasStrings = true;
      capacity += value.length * 3;
    }
    if (buffer.length < capacity) {
      buffer = storage(Math.max(capacity, buffer.length * 2, 256));
      view = new DataView(buffer.buffer);
    }
    view.setUint32(0, 1, true);
    view.setUint32(4, parser == null ? 0 : 1
      | enumeration(parser.sourceType, sourceTypes, "sourceType") << 1
      | enumeration(parser.lang, languages, "lang") << 4
      | boolean(parser.preserveParens, "preserveParens") << 7
      | enumeration(parser.comments, comments, "parser.comments") << 9
      | boolean(parser.semanticErrors, "semanticErrors") << 11, true);
    view.setUint32(8, generator == null ? 0 : 1
      | boolean(generator.asciiOnly, "asciiOnly") << 1
      | boolean(generator.minify, "minify") << 3
      | boolean(generator.inlineScript, "inlineScript") << 5, true);
    view.setUint32(12, boolean(options.sourceMap, "sourceMap")
      | enumeration(options.comments, comments, "comments") << 2, true);
    if (transform == null) {
      buffer.fill(0, 16, 48);
      if (strings[0] == null) return buffer.subarray(0, 48);
      const { written } = encoder.encodeInto(strings[0], buffer.subarray(52));
      view.setUint32(48, written, true);
      buffer.fill(0xff, 52 + written, 84 + written);
      return buffer.subarray(0, 84 + written);
    }
    view.setUint32(16, (transform == null ? 0 : 1) | tsStage << 1 | jsxStage << 3, true);
    view.setUint32(20, (transform?.proposal == null ? 0 : 1)
      | boolean(transform?.proposal?.preventExtensions, "preventExtensions") << 1, true);
    view.setUint32(24, boolean(transform?.features?.[featureKeys[0]], featureKeys[0])
      | boolean(transform?.features?.[featureKeys[1]], featureKeys[1]) << 2, true);
    let low = 0;
    let high = 0;
    const assumptions = transform?.assumptions;
    if (assumptions != null) {
      for (let i = 0; i < 16; i++) low |= boolean(assumptions[assumptionKeys[i]], assumptionKeys[i]) << (i * 2);
      for (let i = 16; i < assumptionKeys.length; i++) high |= boolean(assumptions[assumptionKeys[i]], assumptionKeys[i]) << ((i - 16) * 2);
    }
    view.setUint32(28, low, true);
    view.setUint32(32, high, true);
    view.setUint32(36, boolean(ts?.verbatimModuleSyntax, "verbatimModuleSyntax") << 1
      | boolean(ts?.preserveConstEnums, "preserveConstEnums") << 3
      | enumeration(ts?.jsxRuntime, runtimes, "jsxRuntime") << 5, true);
    view.setUint32(40, enumeration(jsx?.runtime, runtimes, "runtime")
      | boolean(jsx?.development, "development") << 2
      | boolean(jsx?.sourceSelf, "sourceSelf") << 4
      | boolean(jsx?.throwIfNamespace, "throwIfNamespace") << 6
      | boolean(jsx?.pure, "pure") << 8
      | boolean(jsx?.displayName, "displayName") << 10
      | refreshStage << 12, true);
    view.setUint32(44, boolean(refresh?.emitFullSignatures, "emitFullSignatures") << 1, true);
    let offset = 48;
    if (hasStrings) {
      for (const value of strings) {
        if (value == null) {
          view.setUint32(offset, 0xffffffff, true);
          offset += 4;
        } else {
          const { written } = encoder.encodeInto(value, buffer.subarray(offset + 4));
          view.setUint32(offset, written, true);
          offset += 4 + written;
        }
      }
    }
    return buffer.subarray(0, offset);
  };
}

const encodeOptions = createTransformOptionsEncoder();
/** The returned buffer is scratch storage, valid until the next encoding call.
 * Option getters and Proxy traps must not reenter this shared encoder.
 */
export { encodeOptions as encodeTransformOptions };

const utf8Slice = typeof Buffer !== "undefined" && typeof Buffer.prototype.utf8Slice === "function"
  ? Buffer.prototype.utf8Slice
  : function(start, end) { return decoder.decode(this.subarray(start, end)); };
const severities = ["error", "warning", "hint", "info"];

export function decodeTransform(buffer) {
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const codeLength = view.getUint32(0, true);
  const mapLength = view.getUint32(4, true);
  const count = view.getUint32(8, true);
  const code = utf8Slice.call(buffer, 12, 12 + codeLength);
  let offset = 12 + codeLength;
  const map = mapLength === 0xffffffff ? undefined : utf8Slice.call(buffer, offset, offset + mapLength);
  if (map !== undefined) offset += mapLength;
  const diagnostics = new Array(count);
  const readU32 = () => { const value = view.getUint32(offset, true); offset += 4; return value; };
  const readString = () => {
    const length = readU32();
    const value = utf8Slice.call(buffer, offset, offset + length);
    offset += length;
    return value;
  };
  for (let i = 0; i < count; i++) {
    const severity = severities[buffer[offset++]];
    if (severity === undefined) throw new RangeError("Unknown diagnostic severity");
    const start = readU32();
    const end = readU32();
    const message = readString();
    const help = buffer[offset++] === 0 ? undefined : readString();
    const labels = new Array(readU32());
    for (let j = 0; j < labels.length; j++) labels[j] = { start: readU32(), end: readU32(), message: readString() };
    diagnostics[i] = help === undefined ? { severity, message, start, end, labels } : { severity, message, start, end, help, labels };
  }
  return map === undefined ? { code, diagnostics } : { code, map, diagnostics };
}

/** Build a synchronous transform API around a binary native or Wasm binding.
 * Each wrapper owns a non-reentrant options encoder. Option getters and Proxy
 * traps must not reenter the same wrapper while it is encoding options.
 */
export function createTransformSync(binding) {
  const encodeOptions = createTransformOptionsEncoder();
  let sourceBuffer = empty;
  return function transformSync(source, options) {
    if (typeof source !== "string") throw new TypeError("source must be a string");
    const config = encodeOptions(options);
    const capacity = source.length * 3;
    if (sourceBuffer.length < capacity) {
      sourceBuffer = storage(Math.max(capacity, sourceBuffer.length * 2, 4096));
    }
    const { written } = encoder.encodeInto(source, sourceBuffer);
    // Synchronous Rust processing borrows these bytes only for this call.
    const result = binding(sourceBuffer.subarray(0, written), source.length, config);
    return typeof result === "string" ? { code: result, diagnostics: [] } : decodeTransform(result);
  };
}
