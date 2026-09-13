// Runtime support for generated SWC Next WebAssembly bindings.
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
let wasm;
let initialization;

export function initializeSync(input) {
  if (wasm !== undefined) return wasm;
  if (initialization !== undefined) {
    throw new Error("WebAssembly bridge initialization is already in progress");
  }
  if (!isBufferSource(input) && !(input instanceof WebAssembly.Module)) {
    return finalize(input);
  }
  const module = input instanceof WebAssembly.Module ? input : new WebAssembly.Module(input);
  return finalize(new WebAssembly.Instance(module, {}));
}

export function initializeAsync(initialize) {
  if (wasm !== undefined) return Promise.resolve(wasm);
  if (initialization !== undefined) return initialization;
  initialization = Promise.resolve()
    .then(initialize)
    .then(finalize)
    .catch((error) => {
      initialization = undefined;
      throw error;
    });
  return initialization;
}

function finalize(instance) {
  const exports = instance instanceof WebAssembly.Instance ? instance.exports : instance;
  if (exports === null || typeof exports !== "object") {
    throw new TypeError("WebAssembly exports must be an object");
  }
  for (const name of ["memory", "__swc_alloc", "__swc_free"]) {
    if (!(name in exports)) throw new WebAssembly.LinkError(`Missing WebAssembly export: ${name}`);
  }
  wasm = exports;
  initialization = undefined;
  return wasm;
}

export function getFunction(name) {
  const value = initialized()[name];
  if (typeof value !== "function") {
    throw new WebAssembly.LinkError(`Missing WebAssembly export: ${name}`);
  }
  return value;
}

export function withEncodedString(value, callback) {
  return withEncodedBytes(textEncoder.encode(value), callback);
}

export function withEncodedBytes(value, callback) {
  const api = initialized();
  const pointer = api.__swc_alloc(value.length) >>> 0;
  if (pointer === 0) throw new RangeError("WebAssembly allocation failed");
  try {
    new Uint8Array(api.memory.buffer, pointer + 4, value.length).set(value);
    return callback(pointer);
  } finally {
    api.__swc_free(pointer);
  }
}

export function takeBytes(pointer) {
  const api = initialized();
  if (pointer === 0) throw new RangeError("WebAssembly bridge returned a null result");
  try {
    const memory = api.memory.buffer;
    if (pointer > memory.byteLength - 4) {
      throw new RangeError("WebAssembly bridge returned an invalid buffer");
    }
    const length = new DataView(memory).getUint32(pointer, true);
    if (length > memory.byteLength - pointer - 4) {
      throw new RangeError("WebAssembly bridge returned an invalid buffer");
    }
    return new Uint8Array(memory, pointer + 4, length).slice();
  } finally {
    api.__swc_free(pointer);
  }
}

export function takeString(pointer) {
  return decodeString(takeBytes(pointer));
}

export function takeFallibleResult(pointer) {
  const result = takeBytes(pointer);
  if (result.byteLength < 4) {
    throw new RangeError("WebAssembly bridge returned an invalid fallible result");
  }
  const status = new DataView(result.buffer, result.byteOffset, result.byteLength).getUint32(0, true);
  const payload = result.slice(4);
  if (status === 0) return payload;
  if (status === 1) throw new Error(decodeString(payload));
  throw new RangeError("WebAssembly bridge returned an unknown result status");
}

export function decodeString(value) {
  return textDecoder.decode(value);
}

export function decodeU32(value) {
  return dataView(value, 4, "u32").getUint32(0, true);
}

export function decodeI32(value) {
  return dataView(value, 4, "i32").getInt32(0, true);
}

export function decodeF64(value) {
  return dataView(value, 8, "f64").getFloat64(0, true);
}

export function decodeBoolean(value) {
  if (value === 0) return false;
  if (value === 1) return true;
  throw new RangeError("WebAssembly bridge returned an invalid boolean");
}

function dataView(value, length, type) {
  if (value.byteLength !== length) {
    throw new RangeError(`WebAssembly bridge returned an invalid ${type}`);
  }
  return new DataView(value.buffer, value.byteOffset, value.byteLength);
}

function initialized() {
  if (wasm === undefined) throw new Error("WebAssembly bridge is not initialized");
  return wasm;
}

function isBufferSource(value) {
  return value instanceof ArrayBuffer || ArrayBuffer.isView(value);
}
