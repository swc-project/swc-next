import type { Diagnostic } from "./index.js";

export interface TransformOutput {
  code: string;
  map?: string;
  diagnostics: Diagnostic[];
}

/** Scratch storage remains valid until the next call on this encoder.
 * Encoders reuse state and are not reentrant. Option getters and Proxy traps
 * must not reenter the active encoder, including via its transformSync wrapper.
 */
export declare function createTransformOptionsEncoder(
  encodeGeneratorConfig: (options?: object | null) => number,
): (options?: object | null) => Uint8Array;
export declare function decodeTransform(buffer: Uint8Array): TransformOutput;
/** Each wrapper owns a non-reentrant options encoder. Option getters and Proxy
 * traps must not reenter the same wrapper while it is encoding options.
 */
export declare function createTransformSync(
  binding: (source: Uint8Array, sourceUtf16Len: number, options: Uint8Array) => string | Uint8Array,
  encodeGeneratorConfig: (options?: object | null) => number,
): (source: string, options?: object | null) => TransformOutput;
