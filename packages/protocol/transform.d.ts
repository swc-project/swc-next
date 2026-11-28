/** Encode the complete JS transform configuration. The returned scratch buffer
 * is valid until the next call. Option getters and Proxy traps must not reenter
 * this encoder, including via any transformSync entry point.
 * Internal API/format, not covered by semver. Use a matching SWC Next binding.
 */
export declare function encodeTransformApiOptions(
  options: object | null | undefined,
  withPlugins?: boolean,
): Uint8Array;
