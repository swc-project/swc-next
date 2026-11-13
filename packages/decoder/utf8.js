// Read UTF-8 from a Uint8Array without creating a temporary view in Node.js.
export const utf8Slice = typeof Buffer !== "undefined" &&
  typeof Buffer.prototype.utf8Slice === "function"
  ? Buffer.prototype.utf8Slice
  : (() => {
      const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
      return function(start, end) {
        return decoder.decode(this.subarray(start, end));
      };
    })();
