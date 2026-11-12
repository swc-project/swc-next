import {
  parseSync as __parseSync,
  parseBufferSync as __parseBufferSync,
  printSync as __printSync,
  printBuffersSync as __printBuffersSync,
  printBuffersTrustedSync as __printBuffersTrustedSync,
} from "./binding.js";
import {
  decode,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";
import { readAst, NodeView } from "@swc-next/ast";

const textEncoder = new TextEncoder();
const parserViews = new WeakSet();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export { langFromPath, sourceTypeFromPath } from "./binding.js";
export * from "@swc-next/walker";
export { readAst, AstView, NodeView, NodeKind } from "@swc-next/ast";

/** Parse into one self-contained buffer with UTF-8 byte spans and diagnostics. */
export function parseBufferSync(source, options) {
  if (typeof source !== "string") throw new TypeError("Source must be a string");
  return __parseBufferSync(textEncoder.encode(source), encodeParserOptions(options));
}

/** Open a lazy, editable view; ESTree UTF-16 offsets are computed on access. */
export function parseAstSync(source, options) {
  const ast = readAst(parseBufferSync(source, options));
  if (!options?.errorRecovery && ast.diagnostics.length) throw ast.diagnostics[0];
  parserViews.add(ast);
  return ast;
}

export function parseSync(source, options) {
  // Every byte read below is initialized by encodeInto; avoid zeroing the
  // oversized scratch buffer on Node. Keep the portable allocation for browsers.
  const sourceBuffer = typeof source === "string" && typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(source.length * 3).buffer)
    : new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Lazy decoder getters retain sourceBytes, so drop the unused capacity of
  // the encodeInto buffer instead of keeping it alive through a subarray.
  const sourceBytes = canTransferToFixedLength
    ? new Uint8Array(sourceBuffer.buffer.transferToFixedLength(written))
    : sourceBuffer.slice(0, written);
  const buffer = __parseSync(sourceBytes, source.length, encodeParserOptions(options));
  const result = decode(buffer, source, sourceBytes);
  if (!options?.errorRecovery) {
    const diagnostics = result.diagnostics;
    if (diagnostics.length) throw diagnostics[0];
  }
  return result;
}

export function printSync(program, options, meta) {
  if (program instanceof NodeView && program.nodeId === program.ast.rootId) {
    const ast = program.ast;
    // Options may have getters that edit the AST. Finish those calls before
    // borrowing bytes and deciding whether the graph is safe for trusted input.
    const optionBits = encodeCodegenOptions(options);
    const buffers = ast.borrowBuffers(meta);
    if (parserViews.has(ast) && !ast._needsGraphValidation) {
      return { code: __printBuffersTrustedSync(...buffers, optionBits) };
    }
    const code = __printBuffersSync(...buffers, optionBits);
    return { code };
  }
  const code = __printSync(...encode(program, meta), encodeCodegenOptions(options));
  return { code };
}

/** Print an original FFI2 buffer and its edit extension. */
export function printBuffersSync(buffer, extension = new Uint8Array(), options) {
  const code = __printBuffersSync(buffer, extension, encodeCodegenOptions(options));
  return { code };
}
