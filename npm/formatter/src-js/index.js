import {
  formatDocBufferSync,
  printDocBufferSync as printBuffer,
  wasmBinding,
} from "./binding.js";

export { formatDocBufferSync };

export function printDocBufferSync(buffer, options = {}) {
  if (!(buffer instanceof Uint8Array)) {
    throw new TypeError("buffer must be a Uint8Array");
  }
  // Wasm copies input into its memory. Native must first snapshot even shared
  // or externally mutable input before it borrows the bytes.
  let input = buffer;
  if (!wasmBinding) {
    input = new Uint8Array(buffer.byteLength);
    input.set(buffer);
  }
  if (options === null || typeof options !== "object") {
    throw new TypeError("options must be an object");
  }
  const {
    printWidth = 80,
    tabWidth = 2,
    useTabs = false,
    endOfLine = "lf",
  } = options;
  if (!Number.isInteger(printWidth) || printWidth < 0 || printWidth > 65535) {
    throw new RangeError("printWidth must be between 0 and 65535");
  }
  if (!Number.isInteger(tabWidth) || tabWidth < 0 || tabWidth > 64) {
    throw new RangeError("tabWidth must be between 0 and 64");
  }
  if (typeof useTabs !== "boolean") {
    throw new TypeError("useTabs must be a boolean");
  }
  const ending = ["lf", "crlf", "cr"].indexOf(endOfLine);
  if (ending < 0) {
    throw new RangeError("endOfLine must be lf, crlf, or cr");
  }
  const wire = Uint8Array.of(
    0,
    0,
    0,
    0,
    printWidth & 255,
    printWidth >>> 8,
    Number(useTabs),
    tabWidth,
    ending,
  );
  return printBuffer(input, wire);
}
