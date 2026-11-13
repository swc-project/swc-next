import type { Diagnostic } from "./index.js";

export interface TransformOutput {
  code: string;
  map?: string;
  diagnostics: Diagnostic[];
}

/** Scratch storage remains valid until the next call on this encoder. */
export declare function createTransformOptionsEncoder(): (options?: object | null) => Uint8Array;
export declare function encodeTransformOptions(options?: object | null): Uint8Array;
export declare function decodeTransform(buffer: Uint8Array): TransformOutput;
export declare function createTransformSync(
  binding: (source: Uint8Array, sourceUtf16Len: number, options: Uint8Array) => string | Uint8Array,
): (source: string, options?: object | null) => TransformOutput;
