import type { Diagnostic } from "@swc-next/protocol/ast/decode";
import type { Lang, Program, SourceType } from "@swc-next/types";

export type TargetOption =
  | "esnext"
  | "es2022"
  | "es2021"
  | "es2020"
  | "es2019"
  | "es2018"
  | "es2017"
  | "es2016"
  | "es2015";

export type ModuleOption =
  | "preserve"
  | "commonjs"
  | "amd"
  | "umd"
  | "systemjs";

export type JsxRuntimeOption = "automatic" | "classic";
export type DecoratorVersionOption = "2023-11" | "legacy";

export interface AssumptionsOptions {
  constantSuper: boolean;
  ignoreFunctionLength: boolean;
  noDocumentAll: boolean;
  objectRestNoSymbols: boolean;
  privateFieldsAsProperties: boolean;
  pureGetters: boolean;
  setPublicClassFields: boolean;
  setSpreadProperties: boolean;
}

export interface Options {
  parser: {
    lang: Lang;
    sourceType: SourceType;
    preserveParens: boolean;
    semanticErrors: boolean;
  };
  transformer: {
    target: TargetOption;
    module: ModuleOption;
    typescript: {
      enabled: boolean;
      verbatimModuleSyntax: boolean;
    };
    jsx: {
      enabled: boolean;
      runtime: JsxRuntimeOption;
      development: boolean;
      sourceSelf: boolean;
    };
    decorators: {
      enabled: boolean;
      version: DecoratorVersionOption;
      metadata: boolean;
    };
    assumptions: AssumptionsOptions;
  };
  generator: {
    minify: boolean;
    asciiOnly: boolean;
    inlineScript: boolean;
    comments: boolean;
  };
}

export interface PlaygroundState {
  source: string;
  options: Options;
}
export interface CompileRequest extends PlaygroundState {
  id: number;
}
export type CompileResponse =
  | {
      id: number;
      code: string;
      ast: Program;
      diagnostics: Diagnostic[];
      duration: number;
    }
  | { id: number; error: string };
export type CompileResult = Extract<CompileResponse, { code: string }>;
export type SourceRange = { start: number; end: number };
