/** A version-matched SWCD buffer and its opaque label values. */
export interface EncodedDoc {
  buffer: Uint8Array;
  labels: unknown[];
}

/** Encodes a Prettier Doc without loading a native or WASM binding. */
export declare function encodeDoc(doc: unknown): EncodedDoc;
