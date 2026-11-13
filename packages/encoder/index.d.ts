export { encodeGeneratorConfig } from "./options.js";
import type { Program } from "@swc-next/types";

/** Borrowed sections in canonical little-endian order. Consume before calling
 * the same encoder again, and keep them immutable during printing.
 */
export type EncodedAstParts = [
  header: Uint8Array,
  spans: Uint8Array,
  kinds: Uint8Array,
  payloads: Uint8Array,
  extra: Uint8Array,
  strings: Uint8Array,
];

/** Encode an ESTree Program into sections consumed by SWC Next crates.
 * Infer JS/TS from the nodes using the TypeScript-capable encoder.
 * Source type comes from program.sourceType.
 *
 * Encoders reuse module-level state and are not reentrant. AST property getters
 * and Proxy traps must not reenter the active encoder, including via generateSync.
 * encode and encodeTs share the same state.
 */
export declare function encode(program: Program): EncodedAstParts;
export declare function encodeJs(program: Program): EncodedAstParts;
export declare function encodeTs(program: Program): EncodedAstParts;
