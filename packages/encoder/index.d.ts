/** Encode an ESTree Program into the binary format consumed by SWC Next crates.
 * Without a language hint, use the encoder that also supports TypeScript.
 */
export declare function encode(program: any, lang?: "js" | "jsx" | "ts" | "tsx"): Uint8Array;
export declare function encodeJs(program: any): Uint8Array;
export declare function encodeTs(program: any): Uint8Array;
