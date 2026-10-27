import type { Program } from "@swc-next/types";

/** Language and source type associated with an ESTree Program. */
export interface AstMeta {
  readonly lang: "js" | "jsx" | "ts" | "tsx" | "dts";
  readonly sourceType: "module" | "script" | "commonjs" | "unambiguous";
}

/** Transfer sections in canonical little-endian order. Keep them immutable during printing. */
export type EncodedAstParts = [
  header: Uint8Array,
  spans: Uint8Array,
  kinds: Uint8Array,
  payloads: Uint8Array,
  extra: Uint8Array,
  strings: Uint8Array,
];

/** Encode an ESTree Program into sections consumed by SWC Next crates.
 * Pass the decoder result's meta to retain its language and source type.
 * Without metadata, infer JS/TS from the nodes using the TypeScript-capable encoder.
 */
export declare function encode(program: Program, meta?: Partial<AstMeta>): EncodedAstParts;
export declare function encodeJs(program: Program, meta?: Partial<AstMeta>): EncodedAstParts;
export declare function encodeTs(program: Program, meta?: Partial<AstMeta>): EncodedAstParts;
