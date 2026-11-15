/** Encode source as UTF-8 into reusable scratch storage. Consume the returned
 * bytes synchronously before the next call; do not retain or modify them.
 * Internal SWC Next API, not covered by semver.
 */
export declare function encodeSource(source: string): Uint8Array;
