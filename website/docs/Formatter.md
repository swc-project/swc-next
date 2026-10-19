# Formatter design

The formatter targets JavaScript, JSX, TypeScript, and TSX using the SWC Next
AST. Compatibility is defined by printed output and deliberate formatting
choices. Prettier 3.9.9 provides the pinned conformance corpus; later fixes are evaluated
separately. Formatting must preserve program semantics.

## Public contract

`format` parses source with flat comment collection and parenthesis preservation,
returning parse errors before formatting. `format_ast` accepts a program AST
whose caller retains its original source, complete comments, and parenthesized
nodes. Fragment roots are outside this contract.

`FormatOptions` is independent of parser options. Native defaults use tabs and a
120-column width; `FormatOptions::prettier` selects Prettier defaults.
Unsupported syntax returns `FormatError::Unsupported` with its source location.

## Responsibilities and data flow

Source or an existing program AST enters syntax formatting, which constructs a
flat document. Printing interprets that document to produce formatted source.

| Layer | Responsibility |
| --- | --- |
| Entry points | Parser prerequisites, options, and errors |
| Syntax formatting | AST syntax, source boundaries, and layout choices |
| Source and comments | Source ranges and comment ownership |
| Document construction | Stable text and a syntax-independent layout protocol |
| Printing | Width, indentation, line endings, and layout selection |

The printer does not interpret AST nodes or recover missing syntax handling.
Document shape need not match Prettier's internal representation; printed output
is the compatibility boundary.

## Ownership and document protocol

Typed AST handles and borrowed source stay tied to their backing storage.
Document text remains valid until printing finishes. Construction and printing
have separate mutable state.

Generated text belongs to one formatting call and outlives document printing.
It does not retain formatter allocations in the caller's AST.

Doc construction owns flat instruction buffers. Finishing construction transfers
the completed instructions to an immutable document.

Group and indentation scopes form a paired layout protocol. Normal expansion
retains its source preference or required cause, distinct from candidate
measurement. Recorded views retain their syntax scope's layout owner.
Fill entry bounds are relative to the entry, so complete recorded scopes keep
their meaning when viewed or replayed from a different slice origin.

Retained layout candidates freeze complete scopes in storage owned by the
formatting call. Candidates and grouped or ungrouped views borrow the same
immutable instructions, preserving required breaks, suffixes, and named scopes.
Syntax and comments are consumed once, before the printer selects a candidate.

Fit probes use the available width and remaining document without committing
tentative state to printing. A candidate can ignore source-only expansion during
its own measurement while retaining it for rendering. Only the selected layout
affects output, indentation, and line suffixes; syntax and comment ownership
remain unchanged.

## Source and comments

Syntax boundaries give each comment a defined owner. Comments are claimed once
in source order; an unclaimed comment is an explicit formatting error.

Supported annotations and directives retain their effective target and scope;
keeping comment text alone is insufficient. A known absent binding and an
unknown source-bound binding are distinct facts.

Previously inactive annotations may become active; existing effective bindings
must be preserved.

Diagnostic directives retain their source line relationships. Their same-line
and newline constraints take precedence over optional width-dependent breaks.

Prepare real child owners, source token positions, content modes, and line
constraints before consuming the relevant comments. Carry those decisions to
their consumers; claiming a run must not change an already prepared layout
decision.

Ignore targets and physical delimiter ownership are separate. A prepared
boundary retains each real sibling's content mode and following line constraint;
a punctuation slot can retain comments without becoming their target.

Verbatim content retains its selected owner's source interval.

Preserving a protected token gap does not make its adjacent children verbatim:
each child keeps its independently prepared content mode.

A child can return a borrowed pending comment tail when its parent owns the
following punctuation or layout scope. The cursor remains at that run until
the owning boundary claims it.

Statement tails distinguish the current statement, its following sibling, and
an enclosing keyword.

Printed statements forward pending tails beyond their layout group; source
terminators and following keywords consume the runs they own.

## Grammar and semantic boundaries

Expression contexts carry precedence, operand roles, and statement-start rules.
Parenthesis and layout changes must preserve evaluation order, directive status,
statement boundaries, and semantic comment scopes.

Annotation wrappers, their inner types, and annotated bindings remain separate
owners with independent source modes.

Type grammar contexts decide parentheses around real children.

Shared printers receive actual AST children and the caller's grammar context.
The caller retains its enclosing owner, required delimiters, and ending; reuse
does not widen a child's source interval or change its syntax domain.

Required token separation and statement guards are decided by syntax formatting
before printing chooses line breaks or removes optional punctuation.

## Embedded languages

TODO: formatting embedded languages, including HTML, CSS, GraphQL, and other
tagged-template languages, is deferred. Preserve the template's raw text instead
of parsing or reformatting it as an embedded language.

JavaScript and TypeScript interpolation expressions continue through the existing
template-slot formatter, retaining their comment ownership, source delimiters,
and effective directive scopes.

## Verification

Use local fixtures for representative syntax, comments, options, and widths.
The runner checks reparsing, comment retention, and source/AST entry-point
agreement; snapshots expose output and second-pass changes. Parsing and comment
counts alone do not prove semantics: sensitive boundaries need structural or
behavioral checks.

The pinned conformance corpus reports output mismatches, unsupported syntax,
parse errors, panics, and idempotency separately. Unsupported programs remain
visible while syntax support grows.

Document tests verify layout independently of syntax. Report and snapshot
changes are reviewed with their implementation.

Performance comparisons use equivalent inputs, options, and builds.
Formatter-only measurements reuse parsed ASTs; source-to-output measurements
include parsing. Instruction counts and elapsed time are evaluated separately.
