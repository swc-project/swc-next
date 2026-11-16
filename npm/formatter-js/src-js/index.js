import { encodeParserConfig } from "@swc-next/protocol/parser";
import {
  encodeFormatterConfig,
  decodeFormatDocOutput,
} from "@swc-next/protocol/formatter";
import { formatDocBufferSync as formatBuffer } from "./binding.js";

export function formatDocBufferSync(source, parser, options) {
  if (typeof source !== "string")
    throw new TypeError("source must be a string");
  const result = decodeFormatDocOutput(
    formatBuffer(
      source,
      encodeParserConfig(parser),
      encodeFormatterConfig(options),
    ),
    source,
  );
  if (result.error) {
    const error = new SyntaxError(result.error.message);
    error.start = result.error.start;
    error.end = result.error.end;
    throw error;
  }
  return result.buffer;
}
