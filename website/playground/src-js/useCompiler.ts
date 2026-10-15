import { useAtomValue, useSetAtom } from "jotai";
import { useEffect, useRef } from "react";
import { compilerAtom, playgroundAtom } from "./state";
import type { CompileResponse } from "./types";

export function useCompiler() {
  const state = useAtomValue(playgroundAtom);
  const setCompiler = useSetAtom(compilerAtom);
  const worker = useRef<Worker | null>(null);
  const requestId = useRef(0);

  useEffect(() => {
    const compiler = new Worker(
      new URL("./compiler.worker.ts", import.meta.url),
      { type: "module" },
    );
    worker.current = compiler;
    compiler.onmessage = ({ data }: MessageEvent<CompileResponse>) => {
      if (data.id !== requestId.current) return;
      setCompiler(
        "error" in data
          ? { error: data.error, compiling: false }
          : { result: data, compiling: false },
      );
    };
    compiler.onerror = (event) =>
      setCompiler({
        error:
          event.message || "Unable to load the compiler. Reload to try again.",
        compiling: false,
      });
    return () => {
      compiler.terminate();
      worker.current = null;
    };
  }, [setCompiler]);

  useEffect(() => {
    const id = ++requestId.current;
    setCompiler((previous) => ({
      ...previous,
      compiling: true,
      error: undefined,
    }));
    const timeout = window.setTimeout(
      () => worker.current?.postMessage({ id, ...state }),
      180,
    );
    return () => window.clearTimeout(timeout);
  }, [state, setCompiler]);
}
