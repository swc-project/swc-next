# SWC Next security audit

SWC Next is a Rust JavaScript and TypeScript toolchain with a flat, ESTree-based
AST and native JavaScript bindings. It is a separate rewrite of SWC with its own
allocator, AST representation, parser, SIMD code, and bridge implementation.
Its Rust API is still at an early stage. Do not assume SWC's implementation or
security findings apply unchanged to this repository.

## Security model

Apply the build-tool security model described in SWC's security policy:
https://github.com/swc-project/swc/blob/main/SECURITY.md.

SWC Next does not provide a sandbox, resource containment, or guarantees for
processing arbitrary untrusted input. Operators of services accepting untrusted
input must provide input validation, process isolation, and resource limits.
Do not report a missing sandbox or a finding whose only impact depends on an
unsupported, unsandboxed public service deployment.

Use malformed source, ASTs, options, and bridge data to test invariants, but
explain practical confidentiality, integrity, or availability impact in supported
build-tool or Rust API usage. A caller violating an explicitly documented unsafe
API precondition alone is not a vulnerability. Check whether safe APIs or the
JavaScript bindings can reach the same failure without violating their contracts.

## Audit priorities

Prioritize unsafe Rust and the safe code that establishes its invariants:

- `crates/swc_allocator`: allocation layouts, arithmetic overflow, alignment,
  ownership, lifetimes, initialization, and collection operations.
- `crates/swc_ecma_ast`: node storage, typed handles, unchecked access, string
  storage, and mutations that can invalidate references or internal invariants.
- `crates/swc_simd` and parser scanning: vector load bounds, tail handling,
  masks, architecture differences, and fallback behavior.
- `crates/swc_bridge`, `npm/core`, `packages/encoder`, and `packages/decoder`:
  pointer and length validation, buffer lifetimes, UTF-8/UTF-16 conversions,
  wire offsets, and AST round trips across the JavaScript/Rust boundary.
- Parser, codegen, transforms, and other callers that can invalidate the
  assumptions of these unsafe operations.

Include reproducible panics, crashes, excessive memory use, and nontermination
when they have practical availability impact under the security model above.
Distinguish expected syntax diagnostics and ordinary input-proportional resource
use from a security finding. Dependencies and imported test fixtures are not
independent audit targets; show an impact on SWC Next for dependency findings.

## Known issues and deduplication

Exclude the known Rust AST API soundness issue documented under "Rust API
Soundness" in the root README: node IDs and typed handles are not tied to their
owning `Ast`, and using a handle with a different AST can cause undefined behavior
through safe APIs. Do not submit variants with the same root cause as new reports.
Report independent causes or new security impacts, explaining why they differ
from the documented issue. Check current documentation for other known issues.
Group reports by root cause rather than by triggering input or affected caller.

## Reproduction and reporting

The checkout is `/src`. The Dockerfile builds native Rust and Node.js bindings,
fetches the pinned conformance fixtures, and compiles workspace tests. Cargo
dependencies remain available for offline reproduction. Run focused Rust tests
with `cargo test --offline --locked -p <crate>` and Node.js tests with `pnpm test`.
The image does not build or validate the WebAssembly bindings, browser tests, or
non-host architecture paths; state these limits when reporting findings there.

Send all severities privately to `kdy.1997.dev@gmail.com`. For each finding include
the commit, crate or package, minimal input or safe API sequence, exact offline
reproduction command, expected and actual behavior, root cause, and practical
impact. Use memory-checker output or another concrete demonstration for undefined
behavior where possible. For resource exhaustion, measure input size, time, and
memory with bounded reproduction. Explain severity from demonstrated impact and
preconditions; do not infer code execution from a crash alone. Include a focused
regression test and a minimal patch when feasible. Do not publish findings as
public issues.
