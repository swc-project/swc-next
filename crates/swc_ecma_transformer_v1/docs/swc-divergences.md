# SWC Transformer Divergences (ADRs)

`swc_next_ecma_transformer_v1` aims for high compatibility with legacy SWC's transformer, but intentionally diverges where legacy transform behavior reflects outdated ecosystems, deprecated runtimes, or pre-standard tooling.

**Guiding rule:** Intentional divergences from legacy SWC's transformer are introduced strictly for **modernity**—aligning transform defaults and semantics with modern ECMAScript, TypeScript, React, Babel 8, and Node.js standards rather than preserving legacy workarounds by default.

## ADR-001: Modern JSX options (`sourceSelf`, `pure`, `displayName`) and automatic default

- **Legacy SWC transformer:** Defaults JSX to `classic` (`React.createElement`); always injects `__source`/`__self` when `development: true`; unconditionally emits `/*#__PURE__*/` annotations and `createReactClass` `displayName`; exposes historical `useBuiltIns` and `useSpread` flags.
- **v1 transformer:**
  - Public API defaults `jsx.runtime` to `"automatic"` (`react/jsx-runtime`) for omitted `jsx`, `jsx: true`, and `jsx: {}`.
  - Supports independent `sourceSelf` (default `false` in public API), `pure`, and `displayName` (default `false` in public API) switches.
  - Omits `useBuiltIns` and `useSpread` from public options.
- **Rationale:** Aligns with React 19 and Babel 8. Modern React uses the automatic runtime and native Owner Stacks instead of `__source`/`__self` in development, making `sourceSelf: false` a smaller, faster default while keeping opt-in support for React 18 and custom runtimes. `createReactClass` is obsolete, so `displayName` is opt-in.

## ADR-002: TypeScript JSX factory import retention respects preserved JSX and `jsxRuntime`

- **Legacy SWC transformer:** Skips `typescript::tsx` factory import analysis whenever JSX lowering is disabled (`runtime: "preserve"`), causing factory imports such as `import { h } from "preact"` to be erased. Conversely, when JSX lowering is enabled with `runtime: "automatic"`, it still conservatively retains `React` (or configured `pragma`) imports.
- **v1 transformer:** Runs TSX factory import analysis whenever the AST language is TSX, regardless of whether JSX lowering is enabled, and respects `jsxRuntime` (`classic` vs `automatic`) and `@jsxRuntime` pragmas:
  - When JSX is preserved (`jsx: false`) with classic runtime (default), configured `jsxPragma` / `jsxPragmaFrag` imports are retained for downstream transforms.
  - Under `automatic` runtime, unused classic factory imports (such as `import React from "react"`) are erased.
- **Rationale:** Modern pipelines frequently separate TypeScript stripping from JSX lowering (or use React's automatic runtime). TypeScript erasure should neither break downstream classic JSX transforms nor leave dead `React` imports in automatic-runtime output.

## ADR-003: Standard (`2023-11`) decorators as default

- **Legacy SWC transformer:** Defaults to legacy experimental decorators when decorators are enabled without an explicit version.
- **v1 transformer:** Defaults `decorators.version` to `"2023-11"`. Legacy decorators and `emitDecoratorMetadata` remain available via `version: "legacy"`.
- **Rationale:** Matches the TC39 Stage 3 decorator specification and TypeScript 5+ default decorator semantics.

## ADR-004: File-header-only JSX pragma comment scanning

- **Legacy SWC transformer:** JSX lowering scans leading comments on subsequent top-level statements after the first statement until a `@jsx*` directive is found, while TypeScript TSX import retention only scans file-header comments.
- **v1 transformer:** Both TypeScript TSX import retention and JSX lowering only scan file-header leading block comments before the first directive or statement.
- **Rationale:** Matches `tsc` and SWC Next v2, and ensures TypeScript factory import retention and JSX lowering always agree on per-file pragma directives.
