export { encodeParserConfig } from "./options.js";
import type { Program } from "@swc-next/types";

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface DiagnosticLabel {
  message: string;
  /** UTF-16 offset in the source. */
  start: number;
  /** UTF-16 offset in the source. */
  end: number;
}

export interface Diagnostic extends DiagnosticLabel {
  severity: "error" | "warning" | "hint" | "info";
  help?: string;
  labels: DiagnosticLabel[];
}

export interface DecodeResult {
  readonly program: Program;
  readonly comments: Comment[];
  readonly diagnostics: Diagnostic[];
}

/** Decode a buffer produced by a matching SWC Next parser crate into ESTree.
 * Internal use only: this API and format do not follow semver and may change in any release.
 */
export declare function decode(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;

/** Decode a JavaScript buffer produced by a matching SWC Next parser crate.
 * Internal API/format, not covered by semver.
 */
export declare function decodeJs(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;

/** Decode a TypeScript buffer produced by a matching SWC Next parser crate.
 * Internal API/format, not covered by semver.
 */
export declare function decodeTs(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;
