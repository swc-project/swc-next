# Legacy SWC transformer ports

- Match SWC's core behavior and defaults for the options in scope. Keep
  upstream function, type, and field names where they have direct counterparts.
  Add `/// SWC:` links pinned to the upstream commit and source line.
- Adapt the code to SWC Next's AST handles, builders, and resolver. Reuse the
  existing helper injection and hygiene infrastructure.
- Port upstream visitors with `Visit` or `VisitMut`. Use a visitor for a
  single hook and `TraverseMut` for a pass composed of multiple hooks.
- Follow existing Rust option groups and defaults. Expose only the requested
  options and keep serialization out of this crate.
- Wire passes into `Transformer::transform` in SWC's pipeline order. Module
  passes run after compatibility lowering, so their generated syntax must
  respect the target.
- Use the existing SWC compile conformance suite. Add focused regression
  coverage only for behavior the suite misses.
- Pay attention to the code style in [AGENTS.MD](../../AGENTS.md).
- Don't use long full qualified names.
