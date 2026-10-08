import { walk } from "./ast.js";

/** Shared orchestration for native, Node WASM, and browser bindings. */
export function createTransformSync(parseSync, printSync) {
  return function transformSync(source, options = {}) {
    const {
      plugins = [],
      parser: parserOptions,
      codegen: codegenOptions,
    } = options;
    if (!Array.isArray(plugins))
      throw new TypeError("plugins must be an array");
    const parsed = parseSync(source, parserOptions);
    if (
      parsed.diagnostics.some((diagnostic) => diagnostic.severity === "error")
    ) {
      const error = new SyntaxError(
        "Cannot transform source with parser errors",
      );
      error.diagnostics = parsed.diagnostics;
      throw error;
    }
    for (const plugin of plugins) {
      if (!plugin || typeof plugin !== "object" || !plugin.visitor) {
        throw new TypeError(
          "A transform plugin must be an object with visitor hooks; invoke plugin factories before passing them",
        );
      }
      walk(parsed.program, plugin.visitor, plugin.state);
    }
    return {
      ...printSync(parsed.program, codegenOptions, parsed.meta),
      program: parsed.program,
      diagnostics: parsed.diagnostics,
      meta: parsed.meta,
    };
  };
}
