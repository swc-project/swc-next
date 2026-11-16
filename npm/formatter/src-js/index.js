import {
  printDocBufferSync as printBuffer,
  printDocBufferWithCursorSync as printBufferWithCursor,
  wasmBinding,
} from "./binding.js";
import {
  encodePrintConfig,
  decodePrintDocOutput,
} from "@swc-next/protocol/formatter";

export function printDocBufferSync(buffer, options) {
  const config = encodePrintConfig(options);
  return printBuffer(inputBuffer(buffer), config);
}

export function printDocBufferWithCursorSync(buffer, options) {
  const config = encodePrintConfig(options);
  return decodePrintDocOutput(
    printBufferWithCursor(inputBuffer(buffer), config),
  );
}

function inputBuffer(buffer) {
  if (!(buffer instanceof Uint8Array)) {
    throw new TypeError("buffer must be a Uint8Array");
  }
  // Wasm copies input into its memory. Native must snapshot mutable input.
  if (wasmBinding) return buffer;
  const input = new Uint8Array(buffer.byteLength);
  input.set(buffer);
  return input;
}
