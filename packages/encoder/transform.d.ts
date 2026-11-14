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

export interface JsOutput {
  code: string;
  map?: string;
  diagnostics: Diagnostic[];
}

/** Encode source as UTF-8 into reusable scratch storage. Consume the returned
 * bytes synchronously before the next call; do not retain or modify them.
 * Internal SWC Next API, not covered by semver.
 */
export declare function encodeSource(source: string): Uint8Array;

/** Encode the complete JS transform configuration. The returned scratch buffer
 * is valid until the next call. Option getters and Proxy traps must not reenter
 * this encoder, including via any transformSync entry point.
 * The supplied encoders pack the nested parser and generator configurations.
 * Internal API/format, not covered by semver. Use a matching SWC Next binding.
 */
export declare function encodeTransformApiOptions(
  options: object | null | undefined,
  encodeParserConfig: (options?: object | null) => number,
  encodeGeneratorConfig: (options?: object | null) => number,
): Uint8Array;
/** Decode either the native code-only string or a packed native/Wasm result.
 * Internal API/format, not covered by semver. Use a matching SWC Next binding.
 */
export declare function decodeJsOutput(output: string | Uint8Array): JsOutput;
