// Internal SWC Next transport; these APIs and formats do not follow semver.
import { storage } from "./buffer.js";

const encoder = new TextEncoder();
let sourceBuffer = new Uint8Array();

/** Encode source as UTF-8 into reusable scratch storage. Consume the returned
 * bytes synchronously before the next call; do not retain or modify them.
 */
export function encodeSource(source) {
  if (typeof source !== "string") throw new TypeError("source must be a string");
  const capacity = source.length * 3;
  if (sourceBuffer.length < capacity) {
    sourceBuffer = storage(Math.max(capacity, sourceBuffer.length * 2, 4096));
  }
  const { written } = encoder.encodeInto(source, sourceBuffer);
  return sourceBuffer.subarray(0, written);
}
