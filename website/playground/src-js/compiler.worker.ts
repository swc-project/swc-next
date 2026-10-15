import { decode } from "@swc-next/decoder";
import init, { run } from "../wasm/playground.browser.mjs";
import type { CompileRequest, CompileResponse } from "./types";

const ready = init();
// Attach a rejection handler immediately; each request still receives the error.
void ready.catch(() => {});
const encoder = new TextEncoder();
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
    const { program, diagnostics } = decode(
      buffer.subarray(4, 4 + astLength),
      data.source,
      encoder.encode(data.source),
    );
    // Materialize the decoder's lazy getters before crossing the Worker boundary.
    response = {
      id: data.id,
      code: decoder.decode(buffer.subarray(4 + ((astLength + 3) & ~3))),
      ast: structuredClone(program),
      diagnostics,
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
