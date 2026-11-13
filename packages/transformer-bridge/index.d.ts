import type { Diagnostic } from "@swc-next/decoder";

export interface JsOutput {
  code: string;
  map?: string;
  diagnostics: Diagnostic[];
}

/** Encode source as UTF-8 into reusable scratch storage. Consume the returned
 * bytes synchronously before the next call; do not retain or modify them.
 */
export declare function encodeSource(source: string): Uint8Array;

/** Encode the complete JS transform configuration. The returned scratch buffer
 * is valid until the next call. Option getters and Proxy traps must not reenter
 * this encoder, including via any transformSync entry point.
 */
export declare function encodeTransformApiOptions(
  options?: object | null,
): Uint8Array;
/** Decode either the native code-only string or a packed native/Wasm result. */
export declare function decodeJsOutput(output: string | Uint8Array): JsOutput;
