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

/** Decode either the native code-only string or a packed native/Wasm result.
 * Internal API/format, not covered by semver. Use a matching SWC Next binding.
 */
export declare function decodeJsOutput(output: string | Uint8Array): JsOutput;
