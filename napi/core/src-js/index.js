import { parseSync as __parseSync } from "./binding.js";
import { decode } from "./decode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  const program = decode(buffer, source);
  normalizeTypeScriptParameterProperties(program);
  return {
    program,
  };
}

function normalizeTypeScriptParameterProperties(program) {
  const pending = [program];

  while (pending.length > 0) {
    const node = pending.pop();
    if (node.type === "TSParameterProperty") {
      const {
        accessibility,
        decorators,
        override,
        parameter,
        readonly,
        static: isStatic,
        start,
        end,
      } = node;

      // This node is missing from the upstream field-order table used by the
      // ESTree oracle. Preserve typescript-eslint's emitted order here so the
      // binding has stable JSON bytes without changing the SWC Next AST.
      for (const key of Object.keys(node)) {
        if (key !== "type") delete node[key];
      }
      Object.assign(node, {
        accessibility,
        decorators,
        override,
        parameter,
        readonly,
        static: isStatic,
        start,
        end,
      });
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
