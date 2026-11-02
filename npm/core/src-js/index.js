import {
  parseSync as __parseSync,
  parseBufferSync as __parseBufferSync,
  generateSync as __generateSync,
  generateBuffersSync as __generateBuffersSync,
  generateBuffersTrustedSync as __generateBuffersTrustedSync,
} from "./binding.js";
import {
  decode,
  encodeOptions as encodeParserConfig,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeGeneratorConfig,
} from "@swc-next/encoder";
import { readAst, NodeView } from "@swc-next/ast";

const textEncoder = new TextEncoder();
const parserViews = new WeakSet();
let sourceBuffer = new Uint8Array();

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";
export { readAst, AstView, NodeView, NodeKind } from "@swc-next/ast";

/** Parse into one self-contained buffer with UTF-8 byte spans and diagnostics. */
export function parseBufferSync(source, options) {
  if (typeof source !== "string") throw new TypeError("Source must be a string");
  return __parseBufferSync(textEncoder.encode(source), encodeParserConfig(options));
}

/** Open a lazy, editable view; ESTree UTF-16 offsets are computed on access. */
export function parseAstSync(source, options) {
  const ast = readAst(parseBufferSync(source, options));
  if (!options?.errorRecovery && ast.diagnostics.length) throw ast.diagnostics[0];
  parserViews.add(ast);
  return ast;
}

export function parseSync(source, options) {
  const parserConfig = encodeParserConfig(options);
  const capacity = source.length * 3;
  if (sourceBuffer.length < capacity) {
    const length = Math.max(capacity, sourceBuffer.length * 2, 4096);
    sourceBuffer = typeof Buffer === "function"
      ? new Uint8Array(Buffer.allocUnsafeSlow(length).buffer)
      : new Uint8Array(length);
  }
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Native parsing is synchronous. Lazy decoding reads the original string,
  // so this scratch buffer can be overwritten by the next parse.
  const sourceBytes = sourceBuffer.subarray(0, written);
  const buffer = __parseSync(sourceBytes, source.length, parserConfig);
  const result = decode(buffer, source, sourceBytes);
  if (!options?.errorRecovery) {
    const diagnostics = result.diagnostics;
    if (diagnostics.length) throw diagnostics[0];
  }
  return result;
}

export function generateSync(program, options) {
  if (program instanceof NodeView && program.nodeId === program.ast.rootId) {
    const ast = program.ast;
    // Options may have getters that edit the AST. Finish those calls before
    // borrowing bytes and deciding whether the graph is safe for trusted input.
    const generatorConfig = encodeGeneratorConfig(options);
    const buffers = ast.borrowBuffers();
    if (parserViews.has(ast) && !ast._needsGraphValidation) {
      return { code: __generateBuffersTrustedSync(...buffers, generatorConfig) };
    }
    return { code: __generateBuffersSync(...buffers, generatorConfig) };
  }
  const code = __generateSync(...encode(program), encodeGeneratorConfig(options));
  return { code };
}

/** Generate from an original FFI2 buffer and its edit extension. */
export function generateBuffersSync(buffer, extension = new Uint8Array(), options) {
  const code = __generateBuffersSync(buffer, extension, encodeGeneratorConfig(options));
  return { code };
}
