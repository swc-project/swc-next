import { parseSync as __parseSync } from "./binding.js";
import { decode } from "./decode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  const program = decode(buffer, source);
  normalizeRegExpLiteralValues(program);
  return {
    program,
  };
}

function normalizeRegExpLiteralValues(program) {
  const pending = [program];

  while (pending.length > 0) {
    const node = pending.pop();
    // `regex` retains the literal syntax; ESTree's JSON form uses a null value.
    if (node.type === "Literal" && node.regex !== undefined) {
      node.value = null;
    }

    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        for (const item of value) {
          if (item !== null && typeof item === "object") pending.push(item);
        }
      } else if (value !== null && typeof value === "object" && value.type) {
        pending.push(value);
      }
    }
  }
}
