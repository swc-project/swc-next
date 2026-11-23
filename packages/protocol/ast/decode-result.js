const stateKey = Symbol("decode result state");

function readProperty(result, key) {
  const state = result[stateKey];
  const value = state[key];
  return typeof value === "function" ? (state[key] = value()) : value;
}

// Keep the accessors independent of each parse so V8's old-generation
// accessor metadata does not keep the decoder or its cached AST alive.
const descriptors = Object.getOwnPropertyDescriptors({
  get program() {
    return readProperty(this, "program");
  },
  get comments() {
    return readProperty(this, "comments");
  },
  get diagnostics() {
    return readProperty(this, "diagnostics");
  },
});
descriptors[stateKey] = { enumerable: false };

export function createDecodeResult(state) {
  return Object.defineProperties({ [stateKey]: state }, descriptors);
}

export function emptyRecords() {
  return [];
}
