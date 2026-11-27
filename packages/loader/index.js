import { transformSync } from "@swc-next/core";
import { reportDiagnostics, SwcNextLoaderError } from "./diagnostics.js";
import { normalizeSource, resolveTransformOptions } from "./options.js";

export { SwcNextLoaderError };

export function createLoader(customize) {
  return function swcNextLoader(source, inputSourceMap) {
    this.cacheable?.();

    const callback = this.callback;
    try {
      const input = normalizeSource(source);
      let options = resolveTransformOptions(this);
      if (customize?.options) {
        options = customize.options(options, this) ?? options;
      }

      const output = transformSync(input, options);
      const diagnosticError = reportDiagnostics(
        this,
        output.diagnostics,
        input,
        options.filename,
      );
      if (diagnosticError) {
        callback(diagnosticError);
        return;
      }

      const map = output.map !== undefined ? JSON.parse(output.map) : undefined;
      const result = customize?.result
        ? customize.result({ code: output.code, map, inputSourceMap }, this)
        : { code: output.code, map };

      callback(null, result.code, result.map);
    } catch (error) {
      callback(error);
    }
  };
}

const swcNextLoader = createLoader();

export default swcNextLoader;
