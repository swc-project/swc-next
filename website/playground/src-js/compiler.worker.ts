import { readAst } from "@swc-next/ast";
import init, { run } from "../wasm/playground.browser.mjs";
import type { CompileRequest, CompileResponse } from "./types";

const ready = init();
// Attach a rejection handler immediately; each request still receives the error.
void ready.catch(() => {});
const decoder = new TextDecoder();

self.onmessage = async ({ data }: MessageEvent<CompileRequest>) => {
  let response: CompileResponse;
  try {
    await ready;
    const start = performance.now();
    const buffer = run(data.source, JSON.stringify(data.options));
    const astLength = new DataView(buffer.buffer, buffer.byteOffset).getUint32(
      0,
      true,
    );
    const ast = readAst(buffer.subarray(4, 4 + astLength));
    // Copy the tree and lazy diagnostic offsets before crossing the Worker boundary.
    response = {
      id: data.id,
      code: decoder.decode(buffer.subarray(4 + ((astLength + 3) & ~3))),
      ast: ast.materialize(ast.program),
      diagnostics: structuredClone(ast.diagnostics),
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
