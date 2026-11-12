import {
  transformSync as __transformSync,
} from "./binding.js";

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
  const result = decodeTransformOutput(callTransform(__transformSync,
    encodeSource(source),
    source.length,
    JSON.stringify(options ?? {}),
  ));
  throwOnDiagnostics(result.diagnostics, options?.filename ?? "input.js");
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
