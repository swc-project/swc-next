# SWC regular expression parser

This crate is based on
[`oxc_regular_expression` 0.137.0](https://github.com/oxc-project/oxc/tree/6d56d57535f5b81da647e8008f636e0d6cb05f52/crates/oxc_regular_expression),
originally licensed under MIT, with modifications for SWC Next's validation needs.
The parsed regular expression AST uses `swc_next_allocator`, and the crate provides
SWC Next diagnostics and source spans.

The upstream code retains its MIT license; SWC Next contributions are licensed
under Apache-2.0. The Oxc copyright notices and complete MIT license are preserved
in [THIRD_PARTY_NOTICES](../../THIRD_PARTY_NOTICES).
