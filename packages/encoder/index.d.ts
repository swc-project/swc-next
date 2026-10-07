/** Language and source type associated with an ESTree Program. */
export interface AstMeta {
  readonly lang: "js" | "jsx" | "ts" | "tsx" | "dts";
  readonly sourceType: "module" | "script" | "commonjs" | "unambiguous";
}

/** Encode an ESTree Program into the binary format consumed by SWC Next crates.
 * Pass the decoder result's meta to retain its language and source type.
 * Without metadata, infer JS/TS from the nodes using the TypeScript-capable encoder.
 */
export declare function encode(program: any, meta?: Partial<AstMeta>): Uint8Array;
export declare function encodeJs(program: any, meta?: Partial<AstMeta>): Uint8Array;
export declare function encodeTs(program: any, meta?: Partial<AstMeta>): Uint8Array;
