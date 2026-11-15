export function storage(length) {
  return typeof Buffer === "function"
    ? new Uint8Array(Buffer.allocUnsafeSlow(length).buffer)
    : new Uint8Array(length);
}
