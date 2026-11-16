/** JavaScript and TypeScript formatting configuration. */
export interface FormatterConfig {
  /** @default 120 */
  lineWidth?: number;
  /** @default "tab" */
  indentStyle?: "tab" | "space";
  /** @default 2 */
  indentWidth?: number;
  /** @default "lf" */
  lineEnding?: "lf" | "crlf" | "cr";
  semicolons?: "always" | "as-needed";
  quoteStyle?: "double" | "single";
  jsxQuoteStyle?: "double" | "single";
  quoteProperties?: "as-needed" | "consistent" | "preserve";
  trailingCommas?: "all" | "es5" | "none";
  arrowParentheses?: "always" | "as-needed";
  bracketSpacing?: boolean;
  bracketSameLine?: boolean;
  singleAttributePerLine?: boolean;
}

export type { ParserConfig } from "./binding.js";
import type { ParserConfig } from "./binding.js";

/**
 * Produce a version-matched SWCD document without printing it.
 * Throws SyntaxError for source errors, with start/end offsets in UTF-16.
 */
export declare function formatDocBufferSync(
  source: string,
  parser?: ParserConfig | null,
  options?: FormatterConfig | null,
): Uint8Array;
