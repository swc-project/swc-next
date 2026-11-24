/** A version-matched SWCD buffer and its opaque label values. */
export interface EncodedDoc {
  buffer: Uint8Array;
  labels: unknown[];
}

/**
 * Encodes a Prettier Doc without loading a native or WASM binding.
 * Nonnegative alignment widths must be u32 integers. Finite negative alignment
 * values dedent once; -Infinity restores the marked root.
 */
export declare function encodeDoc(doc: unknown): EncodedDoc;

/** Reads the SWCD frame and returns views into the input buffer. */
export declare function readDocument(
  buffer: Uint8Array,
  visitFragment?: (
    view: DataView,
    start: number,
    end: number,
    fragmentCount: number,
  ) => void,
): { stringPool: Uint8Array; labelData: Uint8Array[] };
