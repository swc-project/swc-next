/**
 * DRAFT: proposed configuration API for SWC Next.
 *
 * This file is a design proposal for discussion. It is not exported from the
 * package entry point and has no implementation.
 *
 * Structure:
 *
 * - Top level: settings shared by all languages (`target`, `sourceMap`,
 *   `minify`, `comments`, `format`).
 * - Language sections (`js`, `css`, `html`): may override shared settings when
 *   their meaning differs per language (`target`, `minify`, `comments`), and
 *   contain one section per pipeline stage (`parser`, `transform`, `minify`,
 *   `codegen`) plus `format`.
 * - Each stage section maps to the options of one crate, and the same type is
 *   accepted by the standalone API of that stage (for example `parseSync` and
 *   `printSync`).
 * - Options that affect more than one stage are placed above the stage
 *   sections. Values derived from them are not exposed in stage sections.
 */

import type { WalkHooks } from "@swc-next/walker";
import type { CodegenOptions as BindingCodegenOptions, Lang as JsLang } from "./binding.js";
import type { Diagnostic, ParserOptions as JsParserOptionsStandalone } from "./index.js";

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/** Transforms a source file. Selects the language from `lang` or `filename`. */
export declare function transformSync(source: string, options?: Options): Output;
export declare function transform(source: string, options?: Options): Promise<Output>;

/** Minifies a source file. Equivalent to `transformSync` with `minify: true`. */
export declare function minifySync(source: string, options?: Options): Output;

/** Formats a source file. Reads only `format` settings and `lang`/`filename`. */
export declare function formatSync(source: string, options?: Options): Output;

/**
 * Finds and loads the configuration file for `filename`. Resolves `extends`
 * and applies matching `overrides`. Other APIs never read files implicitly.
 *
 * @returns `null` when no configuration file is found.
 */
export declare function loadConfig(filename: string, options?: LoadConfigOptions): Options | null;

/** Identity function for type checking configuration files. */
export declare function defineConfig(config: ConfigFile): ConfigFile;

/**
 * Converts an SWC `.swcrc` object to SWC Next options. Unsupported keys are
 * reported as warnings. The result maps only to the `js` section.
 */
export declare function fromSwcrc(swcrc: unknown, filename?: string): { options: Options; warnings: Diagnostic[] };

export interface LoadConfigOptions {
  /** Directory where the upward search for a configuration file stops. */
  root?: string;
  /** Explicit configuration file path. Disables the search. */
  configFile?: string;
}

export interface Output {
  code: string;
  /** Source map as a JSON string. `undefined` when source maps are disabled or `sourceMap.emit` is `"inline"`. */
  map?: string;
  /** Legal comments when `comments.legal` is `"external"`. */
  legalComments?: string[];
  /** Diagnostics. The first error is thrown unless error recovery is enabled. */
  diagnostics: Diagnostic[];
}

// ---------------------------------------------------------------------------
// Top level
// ---------------------------------------------------------------------------

/** Input language, including the JavaScript dialect. */
export type Lang = JsLang | "css" | "html";

/** Settings shared by all languages. */
export interface SharedOptions {
  /**
   * Target environments. Controls JavaScript lowering, CSS prefixing and
   * lowering, and syntax available to the minifier and code generator.
   *
   * @default "esnext"
   */
  target?: Target;
  /**
   * Source map generation. `true` is equivalent to `"hidden"`.
   * Applies to all languages and cannot be overridden by a language section.
   *
   * @default false
   */
  sourceMap?: boolean | SourceMapEmit | SourceMapOptions;
  /**
   * Enables minification for all languages. A language section can override
   * this value with `minify`.
   *
   * @default false
   */
  minify?: boolean;
  /**
   * Comment policy for all languages. A language section can override this
   * value with `comments`.
   *
   * @default "all" without minification, "some" with minification
   */
  comments?: CommentsOption;
  /** Layout settings shared by all formatters. */
  format?: LayoutOptions;
}

/** Options accepted by the API. */
export interface Options extends SharedOptions {
  /** Used to infer `lang` and `js.parser.sourceType`, and for diagnostics and source maps. */
  filename?: string;
  /** Input language. Inferred from `filename` when omitted. */
  lang?: Lang;
  /** Return diagnostics and the recovered output instead of throwing the first error. @default false */
  errorRecovery?: boolean;
  js?: JsOptions;
  css?: CssOptions;
  html?: HtmlOptions;
}

/** Contents of a configuration file. */
export interface ConfigFile extends SharedOptions {
  $schema?: string;
  /** Configuration files to extend. Later entries take precedence. */
  extends?: string | string[];
  js?: JsConfigOptions;
  css?: CssOptions;
  html?: HtmlOptions;
  /**
   * File-specific settings. All matching entries are applied in order.
   *
   * Precedence, highest first: override language section, override shared
   * settings, language section, shared settings.
   */
  overrides?: ConfigOverride[];
}

export interface ConfigOverride extends SharedOptions {
  /** Glob patterns matched against the path relative to the configuration file. */
  files: string | string[];
  /** Glob patterns excluded from `files`. */
  excludeFiles?: string | string[];
  js?: JsConfigOptions;
  css?: CssOptions;
  html?: HtmlOptions;
}

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

export type EsVersion =
  | "es5"
  | "es2015"
  | "es2016"
  | "es2017"
  | "es2018"
  | "es2019"
  | "es2020"
  | "es2021"
  | "es2022"
  | "es2023"
  | "es2024"
  | "es2025"
  | "esnext";

export type Engine =
  | "chrome"
  | "edge"
  | "firefox"
  | "safari"
  | "ios"
  | "opera"
  | "samsung"
  | "node"
  | "deno"
  | "bun";

/**
 * An ECMAScript version, browserslist queries, or minimum engine versions.
 *
 * @example "es2022"
 * @example ["chrome >= 100", "safari >= 16"]
 * @example { "node": "20", "chrome": "100" }
 */
export type Target = EsVersion | string[] | Partial<Record<Engine, string>>;

// ---------------------------------------------------------------------------
// Source maps
// ---------------------------------------------------------------------------

/**
 * - `"hidden"`: return the map in `Output.map` without a comment in the code.
 * - `"linked"`: also append `//# sourceMappingURL=<url>` to the code.
 * - `"inline"`: append the map as a data URL. `Output.map` is not returned.
 */
export type SourceMapEmit = "hidden" | "linked" | "inline";

export interface SourceMapOptions {
  /** @default "hidden" */
  emit?: SourceMapEmit;
  /** URL written by `"linked"`. @default `${file}.map` */
  url?: string;

  /** Value of `file`. @default basename of `filename` */
  file?: string;
  /** Value of `sources[0]`. @default `filename` */
  sourceName?: string;
  /** Value of `sourceRoot`. */
  sourceRoot?: string;

  /** Include the original source in `sourcesContent`. @default true */
  sourcesContent?: boolean;
  /** Record original identifier names in `names`. @default true */
  names?: boolean;
  /**
   * Record column positions. `false` records lines only, which lets code
   * generation skip column tracking.
   *
   * @default true
   */
  columns?: boolean;
  /**
   * ECMA-426 `ignoreList`. `true` marks all sources. An array contains glob
   * patterns matched against `sources`.
   *
   * @default false
   */
  ignoreList?: boolean | string[];
  /**
   * Add a `debugId` field and a `//# debugId=` comment. `true` derives the ID
   * from a hash of the output, so it is deterministic.
   *
   * @default false
   */
  debugId?: boolean | string;

  /**
   * Input source map to compose with.
   *
   * - `true`: read an inline data URL in the source. Files are never read.
   * - `false`: ignore input source maps.
   * - string or object: an input map supplied by the caller.
   *
   * An existing `//# sourceMappingURL` comment is removed from the output
   * when source maps are enabled.
   *
   * @default true
   */
  input?: boolean | string | RawSourceMap;
}

export interface RawSourceMap {
  version: 3;
  file?: string;
  sourceRoot?: string;
  sources: (string | null)[];
  sourcesContent?: (string | null)[];
  names: string[];
  mappings: string;
  ignoreList?: number[];
  debugId?: string;
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

/**
 * - `"all"`: keep all comments.
 * - `"some"`: keep legal comments inline, annotations, and comments matching `keep`.
 * - `"none"`: remove all comments.
 *
 * `//# sourceMappingURL` and `//# debugId` comments are controlled by `sourceMap`.
 */
export type CommentsOption = "all" | "some" | "none" | CommentsOptions;

export interface CommentsOptions {
  /** Comments that are not legal comments, annotations, or matched by `keep`. */
  other?: "all" | "none";
  /**
   * Comments starting with `/*!` or containing `@license` or `@preserve`.
   *
   * - `"inline"`: keep at the original position.
   * - `"eof"`: move to the end of the file.
   * - `"external"`: remove from the code and return in `Output.legalComments`.
   * - `"none"`: remove.
   */
  legal?: "inline" | "eof" | "external" | "none";
  /** Keep annotations such as `/*#__PURE__*\/` and `/*@__NO_SIDE_EFFECTS__*\/`. JavaScript only. @default true */
  annotations?: boolean;
  /**
   * Regular expressions for other comments to keep.
   *
   * @example ["webpackChunkName", "@vite-ignore"]
   */
  keep?: string[];
}

// ---------------------------------------------------------------------------
// Formatting layout
// ---------------------------------------------------------------------------

/** Layout settings shared by all language formatters. Maps to `swc_formatter_core::PrintOptions`. */
export interface LayoutOptions {
  /** @default 120 */
  lineWidth?: number;
  /** @default "tab" */
  indentStyle?: "tab" | "space";
  /** Integer in `0..=64`. @default 2 */
  indentWidth?: number;
  /** @default "lf" */
  lineEnding?: "lf" | "crlf" | "cr";
}

// ---------------------------------------------------------------------------
// JavaScript and TypeScript
// ---------------------------------------------------------------------------

export interface JsOptions {
  /** Overrides the shared `target`. Cannot be set per stage. */
  target?: Target;
  /** Overrides the shared `comments`. */
  comments?: CommentsOption;

  /** Parser stage. `lang` comes from the top level; comment collection is derived from `comments`. */
  parser?: JsParserOptions;
  /** Transform stage. */
  transform?: JsTransformOptions;
  /** Minifier stage. `true` uses default options. Overrides the shared `minify`. */
  minify?: boolean | JsMinifyOptions;
  /** Code generation stage. */
  codegen?: JsCodegenOptions;
  /** Formatter. Read only by `formatSync`. */
  format?: JsFormatOptions;

  /** JavaScript plugins. Not serializable, so not allowed in configuration files. */
  plugins?: JsPlugin[];
}

/** JavaScript options allowed in a configuration file. */
export type JsConfigOptions = Omit<JsOptions, "plugins">;

/**
 * Parser options in a pipeline. The same type without `lang` and `comments`
 * is accepted by `parseSync`.
 *
 * In a pipeline, `preserveParens` is chosen per stage, and this value applies
 * only to the standalone `parseSync`.
 */
export type JsParserOptions = Omit<JsParserOptionsStandalone, "lang" | "comments" | "errorRecovery">;

export interface JsTransformOptions {
  /**
   * TypeScript transform. `false` keeps TypeScript syntax in the output.
   *
   * @default enabled when `lang` is `"ts"`, `"tsx"`, or `"dts"`
   */
  typescript?: boolean | TypeScriptOptions;
  /**
   * JSX transform. `false` keeps JSX syntax in the output.
   *
   * @default enabled with the automatic runtime when `lang` is `"jsx"` or `"tsx"`
   */
  jsx?: boolean | JsxOptions;
  /** Decorator transform. @default false */
  decorators?: false | DecoratorOptions;
  /**
   * Per-feature overrides of the lowering selected by `target`. `true` forces
   * the transform, `false` disables it.
   *
   * @example { "class-static-block": true }
   */
  features?: Partial<Record<JsFeature, boolean>>;
  /** Babel-compatible assumptions. Replaces SWC's `loose`. */
  assumptions?: Assumptions;
  /** Module format conversion. @default { type: "preserve" } */
  module?: ModuleOptions;
  /**
   * Replace global expressions with constant expressions.
   *
   * @example { "process.env.NODE_ENV": "\"production\"" }
   */
  define?: Record<string, string>;
  /**
   * - `"inline"`: inject helper functions into each file.
   * - `"external"`: import helpers from `@swc/helpers`.
   *
   * @default "inline"
   */
  helpers?: "inline" | "external";
}

export interface TypeScriptOptions {
  /**
   * TypeScript's `verbatimModuleSyntax`: preserve value imports and exports and
   * disable const enum inlining and removal.
   *
   * @default false
   */
  verbatimModuleSyntax?: boolean;
  /** TypeScript's `preserveConstEnums`: preserve declarations without disabling inlining. @default false */
  preserveConstEnums?: boolean;
}

/**
 * JSX options targeting Babel 8. `useSpread` and `useBuiltIns` are not
 * supported.
 */
export interface JsxOptions {
  /** @default "automatic" */
  runtime?: "automatic" | "classic";
  /** @default false */
  development?: boolean;
  /** Add `__source` and `__self` in development. @default false */
  sourceSelf?: boolean;
  /** Automatic runtime import source. @default "react" */
  importSource?: string;
  /** Classic runtime factory. @default "React.createElement" */
  pragma?: string;
  /** Classic runtime fragment factory. @default "React.Fragment" */
  pragmaFrag?: string;
  /** @default true */
  throwIfNamespace?: boolean;
  /** Mark factory calls as pure. @default true unless `pragma` is set */
  pure?: boolean;
  /** Add `displayName` to `createReactClass` calls. @default false */
  displayName?: boolean;
  /** React Refresh. `true` uses default options. @default false */
  refresh?: boolean | ReactRefreshOptions;
}

export interface ReactRefreshOptions {
  /** @default "$RefreshReg$" */
  refreshReg?: string;
  /** @default "$RefreshSig$" */
  refreshSig?: string;
  /** @default false */
  emitFullSignatures?: boolean;
}

export interface DecoratorOptions {
  /** `"2023-11"` follows the TC39 proposal. `"legacy"` follows TypeScript's `experimentalDecorators`. */
  version: "2023-11" | "legacy";
  /** TypeScript's `emitDecoratorMetadata`. Only with `"legacy"`. @default false */
  metadata?: boolean;
}

/** Names of individually controllable lowering transforms. */
export type JsFeature = "class-static-block" | (string & {});

/** Babel assumptions. All default to `false`. See https://babeljs.io/docs/assumptions. */
export interface Assumptions {
  arrayLikeIsIterable?: boolean;
  constantReexports?: boolean;
  constantSuper?: boolean;
  enumerableModuleMeta?: boolean;
  ignoreFunctionLength?: boolean;
  ignoreFunctionName?: boolean;
  ignoreToPrimitiveHint?: boolean;
  iterableIsArray?: boolean;
  mutableTemplateObject?: boolean;
  noClassCalls?: boolean;
  noDocumentAll?: boolean;
  noIncompleteNsImportDetection?: boolean;
  noNewArrows?: boolean;
  objectRestNoSymbols?: boolean;
  privateFieldsAsProperties?: boolean;
  pureGetters?: boolean;
  setClassMethods?: boolean;
  setComputedProperties?: boolean;
  setPublicClassFields?: boolean;
  setSpreadProperties?: boolean;
  skipForOfIteratorClosing?: boolean;
  superIsCallableConstructor?: boolean;
}

/** Module format conversion. Scope is still to be decided. */
export interface ModuleOptions {
  type: "preserve" | "commonjs";
}

export interface JsMinifyOptions {
  /** `false` disables compression. `true` uses default options. @default true */
  compress?: boolean | JsCompressOptions;
  /** `false` disables mangling. `true` uses default options. @default true */
  mangle?: boolean | JsMangleOptions;
}

/** All default to `true`. */
export interface JsCompressOptions {
  /** Remove empty statements and unnecessary blocks. */
  blocks?: boolean;
  /** Shorten `true` to `!0` and `false` to `!1`. */
  booleans?: boolean;
  /** Remove code that a constant condition never runs. */
  deadCode?: boolean;
  /** Fold constant expressions when the result is shorter. */
  evaluate?: boolean;
  /** Shorten unshadowed global constants. */
  globalDefs?: boolean;
  /** Join adjacent declarations of the same kind. */
  joinVars?: boolean;
  /** Join adjacent expression statements with the comma operator. */
  sequences?: boolean;
  /** Shorten undefined `typeof` comparisons for local `var` bindings. */
  typeofs?: boolean;
  /** Remove unused local variables whose initializers have no side effects. */
  unused?: boolean;
}

export interface JsMangleOptions {
  /** Mangle top-level names. @default false */
  topLevel?: boolean;
  /** Keep class names. @default false */
  keepClassNames?: boolean;
  /** Keep function names. @default false */
  keepFnNames?: boolean;
}

/**
 * Code generation options. The same type is accepted by `printSync`.
 *
 * `minify` defaults to the resolved `minify` setting. Comment output is
 * controlled by `comments`, and source maps by `sourceMap`.
 */
export interface JsCodegenOptions extends BindingCodegenOptions {
  /** Text inserted at the start of the output. */
  preamble?: string;
}

/** JavaScript-specific formatting options. Maps to `swc_ecma_formatter::FormatOptions`. */
export interface JsFormatOptions extends LayoutOptions {
  /** @default "always" */
  semicolons?: "always" | "asNeeded";
  /** @default "double" */
  quoteStyle?: "double" | "single";
  /** @default "double" */
  jsxQuoteStyle?: "double" | "single";
  /** @default "asNeeded" */
  quoteProperties?: "asNeeded" | "consistent" | "preserve";
  /** @default "all" */
  trailingCommas?: "all" | "es5" | "none";
  /** @default "always" */
  arrowParentheses?: "always" | "asNeeded";
  /** @default true */
  bracketSpacing?: boolean;
  /** @default false */
  bracketSameLine?: boolean;
  /** Put each JSX attribute on its own line when an element has multiple attributes. @default false */
  singleAttributePerLine?: boolean;
}

export interface JsPlugin {
  name: string;
  /**
   * When the visitor runs.
   *
   * - `"afterParse"`: before the transform stage.
   * - `"beforeCodegen"`: after the transform and minifier stages.
   *
   * @default "afterParse"
   */
  stage?: "afterParse" | "beforeCodegen";
  visitor: WalkHooks;
}

// ---------------------------------------------------------------------------
// CSS (placeholder)
// ---------------------------------------------------------------------------

export interface CssOptions {
  target?: Target;
  comments?: CommentsOption;
  parser?: CssParserOptions;
  transform?: CssTransformOptions;
  minify?: boolean | CssMinifyOptions;
  codegen?: CssCodegenOptions;
  format?: CssFormatOptions;
}

export interface CssParserOptions {}

export interface CssTransformOptions {
  /** CSS Modules. `true` uses default options. @default false */
  modules?: boolean | { pattern?: string };
}

export interface CssMinifyOptions {}

export interface CssCodegenOptions {}

export interface CssFormatOptions extends LayoutOptions {}

// ---------------------------------------------------------------------------
// HTML (placeholder)
// ---------------------------------------------------------------------------

/**
 * Embedded `<script>` and `<style>` content uses the `js` and `css` sections.
 * The dialect of embedded scripts comes from the element, such as `lang="ts"`.
 */
export interface HtmlOptions {
  comments?: CommentsOption;
  parser?: HtmlParserOptions;
  minify?: boolean | HtmlMinifyOptions;
  codegen?: HtmlCodegenOptions;
  format?: HtmlFormatOptions;
}

export interface HtmlParserOptions {}

export interface HtmlMinifyOptions {
  /** @default "conservative" */
  collapseWhitespace?: "none" | "conservative" | "smart";
}

export interface HtmlCodegenOptions {}

export interface HtmlFormatOptions extends LayoutOptions {}
