export { encodeGeneratorConfig } from "./options.js";
import type { Program } from "@swc-next/types";

/** Borrowed sections in canonical little-endian order. Consume before calling
 * the same encoder again, and keep them immutable during printing.
 * Internal SWC Next transport; the layout does not follow semver.
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
 * Internal use only: this API and format do not follow semver and may change
 * in any release. Use a matching SWC Next native/Wasm consumer.
 *
 * Encoders reuse module-level state and are not reentrant. AST property getters
 * and Proxy traps must not reenter the active encoder, including via generateSync.
 * encode and encodeTs share the same state.
 */
export declare function encode(program: Program): EncodedAstParts;
/** JavaScript-only variant of encode; internal API/format, not covered by semver. */
export declare function encodeJs(program: Program): EncodedAstParts;
/** TypeScript-capable variant of encode; internal API/format, not covered by semver. */
export declare function encodeTs(program: Program): EncodedAstParts;
