import type { Diagnostic, TransformApiOptions } from "@swc-next/core";

export type SwcNextLoaderOptions = TransformApiOptions;

export interface RawSourceMap {
  version: number;
  sources: string[];
  names: string[];
  mappings: string;
  file?: string;
  sourceRoot?: string;
  sourcesContent?: Array<string | null>;
}

export class SwcNextLoaderError extends Error {
  readonly diagnostics: Diagnostic[];
  constructor(message: string, diagnostics?: Diagnostic[]);
}

export interface LoaderContext<Options = SwcNextLoaderOptions> {
  readonly resourcePath: string;
  readonly sourceMap: boolean;
  cacheable?: (flag?: boolean) => void;
  getOptions?: () => Options;
  emitWarning?: (warning: Error) => void;
  callback: (
    err: Error | null | undefined,
    content?: string | Buffer,
    sourceMap?: string | RawSourceMap,
  ) => void;
}

export interface LoaderResult {
  code: string;
  map?: RawSourceMap;
  inputSourceMap?: string | RawSourceMap | null;
}

export interface LoaderCustomizer<Options = SwcNextLoaderOptions> {
  options?: (
    options: Options,
    context: LoaderContext<Options>,
  ) => SwcNextLoaderOptions | void;
  result?: (
    result: LoaderResult,
    context: LoaderContext<Options>,
  ) => { code: string; map?: string | RawSourceMap };
}

export type SwcNextLoader<Options = SwcNextLoaderOptions> = (
  this: LoaderContext<Options>,
  source: string | Buffer,
  inputSourceMap?: string | RawSourceMap | null,
) => void;

export declare function createLoader<Options = SwcNextLoaderOptions>(
  customize?: LoaderCustomizer<Options>,
): SwcNextLoader<Options>;

declare const swcNextLoader: SwcNextLoader<SwcNextLoaderOptions>;

export default swcNextLoader;
