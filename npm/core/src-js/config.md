# Configuration API draft: design notes

This document explains the design decisions in [`config.d.ts`](./config.d.ts),
with the advantages and disadvantages of each decision. It is a proposal for
discussion. Nothing in it is implemented.

## Problems with `.swcrc`

| Problem | Example |
| --- | --- |
| Nesting without meaning | `jsc.*`, `jsc.transform.react` |
| More than one option for the same thing | `jsc.target` and `env.targets`; `loose` and `assumptions`; top-level `minify`, `jsc.minify`, and `jsc.minify.format` |
| Parser flags that switch syntax on and off | `syntax` with `tsx`, `jsx`, `decorators`, `dynamicImport`, and others |
| Two ways to set the module kind | `isModule: true \| false \| "unknown" \| "commonjs"` |
| Implicit file reads | `swcrc: true` is the default, so callers such as Rspack must disable it |
| File matching with regular expressions | `test` and `exclude`; in an array, only the first matching entry applies |
| Options that are not compiler tasks | `jsc.baseUrl`, `jsc.paths` |
| ES5 as the default target | Unnecessary lowering for most current environments |
| Source map options in many places | `sourceMaps`, `inputSourceMap`, `inlineSourcesContent`, `sourceRoot`, `sourceFileName`, `outputPath`, `emitSourceMapColumns`, `jsc.minify.sourceMap` |
| JavaScript only | No structure for CSS or HTML |

## Structure

```
top level:  target, sourceMap, minify, comments, format (layout)
  js / css / html:
    overrides:  target?, minify?, comments?
    stages:     parser, transform, minify, codegen
    tool:       format
```

## Decisions

### 1. Shared settings at the top level, language sections below

The top level contains settings that apply to all languages. Each language has
its own section. An alternative is to group by tool first (`minify.js`,
`minify.css`).

Advantages:

- HTML embeds JavaScript and CSS. Embedded `<script>` and `<style>` content
  uses the `js` and `css` sections directly. With tool-first grouping, the HTML
  pipeline must collect settings from several tool sections.
- Some values are shared across languages. `target` controls JavaScript
  lowering and CSS prefixing. Layout settings map to
  `swc_formatter_core::PrintOptions`, which all language printers use.
- Users who need only one language read only one section.
- The Rust configuration crate can enable each language with a Cargo feature.
  Rspack can use JavaScript without a dependency on CSS or HTML crates.

Disadvantages:

- A setting that applies to all languages for one tool, such as "disable
  mangling everywhere", needs one entry per language.
- Shared settings and language overrides use the same keys. Users must know the
  precedence rules.

### 2. One section per pipeline stage in each language

Each language section has `parser`, `transform`, `minify`, and `codegen`
sections. Each section maps to the options of one crate.

Advantages:

- Each option has a clear owner crate. Normalization is almost a direct copy.
- The standalone API of each stage accepts the same type. A JavaScript plugin
  flow can pass configuration sections directly:

  ```ts
  const { program } = parseSync(source, { ...config.js.parser, lang });
  walk(program, visitor);
  printSync(program, config.js.codegen);
  ```

- CSS and HTML use the same shape.
- The names match existing APIs (`CodegenOptions`, `printSync`).
- The bridge already has `#[bridge(object)]` types for the parser and codegen.
  Transform and minify types can follow the same pattern.

Disadvantages:

- Paths become deep, for example `js.transform.jsx.refresh`. This is the same
  depth as `jsc.transform.react.refresh`, but each level has a meaning.
- Users must know which stage owns an option. For example, `define` is in
  `transform`, but a user can expect it in `minify`.
- Some options affect more than one stage. Decision 3 handles these options.

Alternative: move the `transform` keys directly into `js`. This makes paths
shorter, but a section no longer equals the options type of one standalone API.

### 3. Options that affect more than one stage are above the stage sections

| Option | Affected stages | Location | Rule |
| --- | --- | --- | --- |
| `lang` | parser, transform, codegen | API argument, inferred from `filename` | Not in a section. It is a property of each input, and embedded HTML scripts have their own `lang`. |
| `target` | transform, minify, codegen | Top level, language override | Cannot be set per stage. |
| `sourceMap` | parser (input map), codegen | Top level only | See decision 4. |
| `comments` | parser (collection), codegen (output) | Top level, language override | Parser collection is derived. See decision 5. |
| `minify: true` | minify, `codegen.minify` | Top level, language override | Sets the defaults of both. |
| `preserveParens` | parser | `js.parser` | In a pipeline, the value is chosen per stage. The user value applies only to `parseSync`. |

Advantages:

- Contradictory combinations cannot occur, or cause an error when they do.
- Users do not need to know the internal stage order for common settings.

Disadvantages:

- The type of a stage section in a configuration file is a subset of the
  standalone type (`Omit<ParserOptions, "lang" | "comments">`). The two types
  are similar but not equal.
- Derived values are invisible. Users who read only a stage section can miss
  the reason for a behavior.

### 4. `sourceMap` at the top level only

Source map options are grouped by purpose: how the map is attached (`emit`,
`url`), what it contains (`sourcesContent`, `names`, `columns`, `ignoreList`,
`debugId`), paths (`file`, `sourceName`, `sourceRoot`), and input maps
(`input`).

Advantages:

- No source map option has a different meaning per language. One location is
  sufficient, and `overrides` handles file-specific settings.
- The common cases need one string: `sourceMap: "linked"`.
- `columns: false` and `names: false` reduce cost. Code generation can skip
  column tracking in development builds.
- `input: true` reads only inline data URLs. The API does not read files.
- When transform and minify run in one pipeline, there is one map and no
  intermediate composition. `jsc.minify.sourceMap` is not necessary.
- HTML returns one map that includes embedded scripts and styles.

Disadvantages:

- An external `.map` input file must be read by the caller, `loadConfig`, or
  the CLI. With SWC, `inputSourceMap: true` read it automatically.
- There is no path transform callback (Rollup's `sourcemapPathTransform`).
  Callers must edit `map.sources` after the call.
- `input: true` trusts inline maps in the source by default.

The Rust API returns the `SourceMap` structure without JSON serialization.
`compose` and `finalize` (serialization and comments) are separate steps. These
settings are for the JavaScript API and the CLI.

### 5. `comments` at the top level, with language overrides

`comments` controls which comments the output keeps. It accepts `"all"`,
`"some"`, `"none"`, or an object with `other`, `legal`, `annotations`, and
`keep`.

Advantages:

- CSS and HTML also need a comment policy, so the default is shared.
- JavaScript has annotations (`/*#__PURE__*/`) that other languages do not
  have, so `js.comments` can override the default.
- Legal comments, annotations, and bundler magic comments
  (`webpackChunkName`, `@vite-ignore`) can be handled separately.
- `legal: "external"` is simple to implement. Codegen already returns
  `legal_comments` separately.
- `js.parser` has no `comments` key. In a configuration file, `comments`
  appears only at the top level and in language sections, with the same
  meaning in both.

Disadvantages:

- `parseSync` keeps `comments: "none" | "flat"`, which selects how the AST
  collects comments. The same name has a different meaning and value set in the
  configuration. Documentation must explain the difference. If this causes
  confusion, `parseSync` can rename it later (for example `commentMode`), but
  that is a breaking change to a published API.
- `keep` takes regular expression strings. Validation and error messages for
  invalid patterns are necessary.

### 6. The parser receives intent, not syntax flags

The parser receives only `lang` and `sourceType`. Both are inferred from
`filename` (`langFromPath` and `sourceTypeFromPath` already exist). The parser
always accepts standard syntax.

Advantages:

- Historical flags such as `dynamicImport` and `topLevelAwait` are removed.
- One `lang` value replaces `syntax`, `tsx`, `jsx`, and `dts`.
- One `sourceType` value replaces `isModule`.

Disadvantages:

- Users cannot reject a specific syntax at parse time. A linter must do that.
- Proposal syntax that is not standard needs a separate decision when it is
  added.

### 7. One `target`, per-feature overrides, and assumptions only

`target` accepts an ECMAScript version, browserslist queries, or engine
versions. `js.transform.features` overrides single transforms. `assumptions`
replaces `loose`.

Advantages:

- The conflict between `jsc.target` and `env.targets` cannot occur.
- `features` is a record, so a feature cannot be both included and excluded,
  which was possible with `env.include` and `env.exclude`.
- `assumptions` describes each relaxation explicitly. `loose` had a different
  meaning in each transform.

Disadvantages:

- Users who depend on `loose` must find the equivalent assumptions.
  `fromSwcrc` can report the mapping.
- Feature names are part of the public API and must stay stable.
- Polyfill injection (`env.mode`, `coreJs`) is not included. It needs a
  separate option later.

### 8. No implicit file reads

`transformSync` and the other APIs use only the options they receive.
`loadConfig` finds and loads configuration files. `ConfigFile` (with `extends`
and `overrides`) and `Options` (for the API) are separate types.

Advantages:

- No file system access in the hot path. Callers such as Rspack do not need to
  disable configuration loading.
- The API works the same in Node.js, WASI, and browsers.
- `overrides` uses glob patterns, and all matching entries are applied in
  order, like ESLint and Biome. With `.swcrc`, only the first matching entry
  applies.

Disadvantages:

- Users who expect `.swcrc` to load automatically must call `loadConfig`.
- Two related types (`ConfigFile` and `Options`) must stay consistent.

### 9. Defaults that do the least work

`target` defaults to `"esnext"`. TypeScript and JSX transforms are enabled
according to `lang`.

Advantages:

- Most current environments need no lowering.
- A TypeScript or JSX input produces JavaScript without extra settings.

Disadvantages:

- SWC users who expect ES5 output get different output without a warning.
  `fromSwcrc` must set `target` explicitly.
- When TypeScript is enabled by default, keeping TypeScript syntax in the
  output needs `typescript: false`.

### 10. Compatibility through `fromSwcrc`

`fromSwcrc` converts an `.swcrc` object to new options and reports unsupported
keys as warnings. The test suite mapping in
`tasks/testsuite/src/cases/swc_transform/options.rs` moves into this function.

Advantages:

- The new API does not keep old names for compatibility.
- Migration is explicit, and warnings show what is lost.
- The test suite uses the same conversion as users.

Disadvantages:

- The conversion must be maintained while users migrate.
- Some `.swcrc` options have no equivalent (`jsc.paths`, `env.mode`).

### 11. Plugins only in the API

`js.plugins` accepts walker hooks and runs them after parsing or before code
generation. Configuration files cannot contain plugins.

Advantages:

- Functions are not serializable, so the configuration file stays plain JSON.
- Plugins use the existing `walk` API.

Disadvantages:

- Plugins cannot be enabled from a configuration file. A JavaScript
  configuration file (`swc.config.ts`) would solve this but adds a loader.
- WebAssembly plugins need a separate design.

### 12. One entry point per tool, one parse API per language

`transformSync`, `minifySync`, and `formatSync` select the language from `lang`
or `filename`. Parse and print APIs stay per language because AST types differ
(`@swc-next/core`, later `@swc-next/core/css` and `@swc-next/core/html`).
`Output` has the same type for all languages.

Advantages:

- Tools that process many file types call one function.
- AST types stay precise for each language.

Disadvantages:

- The package must include all languages, or the entry point must report an
  error for a language that is not available.

## Mapping from `.swcrc`

| `.swcrc` | New option |
| --- | --- |
| `jsc.parser.syntax`, `tsx`, `jsx`, `dts` | `lang` (or inferred from `filename`) |
| `jsc.parser.decorators`, `dynamicImport`, and similar | Removed |
| `isModule` | `js.parser.sourceType` |
| `jsc.target`, `env.targets` | `target` |
| `env.include`, `env.exclude` | `js.transform.features` |
| `jsc.loose`, `env.loose` | `js.transform.assumptions` |
| `jsc.transform.react` | `js.transform.jsx` |
| `jsc.transform.legacyDecorator`, `decoratorMetadata` | `js.transform.decorators` |
| `jsc.transform.verbatimModuleSyntax` | `js.transform.typescript.verbatimModuleSyntax` |
| `jsc.externalHelpers` | `js.transform.helpers` |
| `minify`, `jsc.minify.compress`, `jsc.minify.mangle` | `minify`, `js.minify` |
| `jsc.minify.format`, `jsc.output.charset` | `js.codegen` |
| `jsc.keepClassNames` | `js.minify.mangle.keepClassNames` |
| `jsc.preserveAllComments` | `comments` |
| `sourceMaps` | `sourceMap` |
| `inputSourceMap` | `sourceMap.input` |
| `inlineSourcesContent` | `sourceMap.sourcesContent` |
| `sourceFileName` | `sourceMap.sourceName` |
| `sourceRoot` | `sourceMap.sourceRoot` |
| `outputPath` | `sourceMap.file`, `sourceMap.url` |
| `emitSourceMapColumns` | `sourceMap.columns` |
| `jsc.minify.sourceMap` | Removed |
| `test`, `exclude` | `overrides[].files`, `overrides[].excludeFiles` |
| `jsc.baseUrl`, `jsc.paths` | Removed |
| `env.mode`, `env.coreJs` | Not included |
| `jsc.experimental.plugins` | `js.plugins` (WebAssembly plugins later) |

## Open questions

1. `typescript: false`: keep TypeScript syntax in the output (current draft),
   or reject TypeScript input?
2. Configuration file name: `swc.config.json` (recommended, avoids conflicts
   with SWC) or `.swcrc` with the new schema? Support `swc.config.ts`?
3. `target` default: `"esnext"` (current draft) or `"es5"` like SWC?
4. `sourceMap: true`: `"hidden"` (current draft, suits API users) or
   `"linked"` (suits CLI users)? The CLI can use a different default.
5. `sourceMap.input` default: `true` (current draft, like Babel) or `false`?
6. Emit `x_google_ignoreList` in addition to the standard `ignoreList`?
7. Reserve a name for the TG4 scopes proposal now?
8. Top-level `lang`: one extended union (current draft) or a language plus a
   dialect (`language: "js"` with `js.parser.lang: "tsx"`)?
9. Section names: `js`, `css`, `html` (current draft) or `javascript` like
   Biome?
10. Reserve `lang` values for SCSS, Less, JSON, and Markdown?
11. Scope of module conversion (`commonjs`, `amd`, `umd`, `systemjs`) in the
    first version?
