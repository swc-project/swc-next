import { decode } from "@swc-next/protocol/ast/decode";
import { decodeJsOutput } from "@swc-next/protocol/output";
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { encodeSource } from "@swc-next/protocol/source";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import init, { parseSync, transformSync } from "../wasm/playground.browser.mjs";
import type { CompileRequest, CompileResponse, Options } from "./types";

const ready = init();
// Attach a rejection handler immediately; each request still receives the error.
void ready.catch(() => {});

function toApiOptions(options: Options) {
  const lang = options.parser.lang;
  const filename = lang === "dts" ? "input.d.ts" : `input.${lang}`;
  const isTs = ["ts", "tsx", "dts"].includes(lang);
  const isJsx = ["jsx", "tsx"].includes(lang);
  const { target, module, typescript, jsx, decorators, assumptions } =
    options.transformer;
  const activeAssumptions = Object.fromEntries(
    Object.entries(assumptions).filter(([, value]) => value),
  );
  return {
    filename,
    comments: options.generator.comments ? "all" : "none",
    parser: {
      lang,
      sourceType: options.parser.sourceType,
      preserveParens: options.parser.preserveParens,
      semanticErrors: options.parser.semanticErrors,
    },
    transformer: {
      target,
      module: { type: module },
      typescript: isTs
        ? typescript.enabled
          ? { verbatimModuleSyntax: typescript.verbatimModuleSyntax }
          : false
        : false,
      jsx: isJsx
        ? jsx.enabled
          ? {
              runtime: jsx.runtime,
              development: jsx.development,
              sourceSelf: jsx.development && jsx.sourceSelf,
            }
          : false
        : false,
      decorators: decorators.enabled
        ? {
            version: decorators.version,
            metadata:
              decorators.version === "legacy" ? decorators.metadata : false,
          }
        : false,
      ...(Object.keys(activeAssumptions).length > 0
        ? { assumptions: activeAssumptions }
        : {}),
    },
    generator: {
      minify: options.generator.minify,
      asciiOnly: options.generator.asciiOnly,
      inlineScript: options.generator.inlineScript,
    },
  };
}

self.onmessage = async ({ data }: MessageEvent<CompileRequest>) => {
  let response: CompileResponse;
  try {
    await ready;
    const start = performance.now();
    const sourceBytes = encodeSource(data.source);
    const apiOptions = toApiOptions(data.options);
    const transformed = decodeJsOutput(
      transformSync(
        sourceBytes,
        data.source.length,
        encodeTransformApiOptions(apiOptions),
      ),
    );
    const { program } = decode(
      parseSync(
        sourceBytes,
        data.source.length,
        encodeParserConfig(apiOptions.parser),
      ),
      data.source,
    );
    // Materialize the decoder's lazy getters before crossing the Worker boundary.
    response = {
      id: data.id,
      code: transformed.code,
      ast: structuredClone(program),
      diagnostics: transformed.diagnostics,
      duration: performance.now() - start,
    };
  } catch (error) {
    response = {
      id: data.id,
      error: error instanceof Error ? error.message : String(error),
    };
  }
  self.postMessage(response);
};
