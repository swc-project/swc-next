export function resolveTransformOptions(context) {
  const rawOptions =
    typeof context.getOptions === "function" ? context.getOptions() : undefined;
  const options =
    rawOptions && typeof rawOptions === "object" ? { ...rawOptions } : {};

  if (options.filename === undefined && context.resourcePath) {
    options.filename = context.resourcePath;
  }
  if (options.sourceMap === undefined) {
    options.sourceMap = Boolean(context.sourceMap);
  }

  return options;
}

export function normalizeSource(source) {
  if (typeof source === "string") return source;
  if (Buffer.isBuffer(source)) return source.toString("utf8");
  if (source instanceof Uint8Array) {
    return Buffer.from(source.buffer, source.byteOffset, source.byteLength).toString("utf8");
  }
  throw new TypeError("Expected source to be a string or Buffer");
}
