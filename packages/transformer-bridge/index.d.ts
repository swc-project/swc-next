import type { Diagnostic } from "@swc-next/decoder";

export interface TransformOutput {
  code: string;
  map?: string;
  diagnostics: Diagnostic[];
}

/** Encode the complete JS transform configuration. The returned scratch buffer
 * is valid until the next call. Option getters and Proxy traps must not reenter
 * this encoder while it is encoding.
 */
export declare function encodeJsTransformApiOptions(
  options?: object | null,
): Uint8Array;
/** Decode either the native code-only string or a packed native/Wasm result. */
export declare function decodeJsTransformOutput(output: string | Uint8Array): TransformOutput;
/** Each wrapper owns a non-reentrant options encoder. Option getters and Proxy
 * traps must not reenter the same wrapper while it is encoding options.
 */
export declare function createTransformSync(
  binding: (source: Uint8Array, sourceUtf16Len: number, options: Uint8Array) => string | Uint8Array,
): (source: string, options?: object | null) => TransformOutput;
