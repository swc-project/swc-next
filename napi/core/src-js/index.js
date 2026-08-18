import { parseSync as __parseSync } from "./binding.js";
import { decode } from "./decode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  let { program, decodeComments } = decode(buffer, source);
  let comments;
  return {
    program,
    get comments() {
      // Materialize once, cache the array, then release the transfer closure.
      if (comments === undefined) {
        comments = decodeComments();
        decodeComments = undefined;
      }
      return comments;
    },
  };
}
