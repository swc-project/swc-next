# Formatter Invariants

Match the pinned Prettier version with equivalent options unless these invariants
require otherwise. These are requirements, not a claim that all inputs satisfy them.

## Program meaning

Preserve evaluation order, statement and optional-chain boundaries, type meaning,
significant JSX whitespace, and tagged-template raw contents.

Preserve JavaScript directive prologues in both directions: neither disable nor
activate directives. Keep the scope, barriers, and quote/escape spelling that affect
activation.

Source and AST entry points must agree when their documented preconditions hold.
Reparsing alone does not prove semantic equivalence.

## Comments and directives

Emit each comment exactly once. Preserve its content and relevant ownership and
target relationships; source position, printing owner, and consumer target differ.
Ordinary blank lines, absolute positions, and AST attachment fields are not inherently
fixed.

Within the documented scope, preserve active tool directives and their coverage,
including TypeScript diagnostics, optimizer annotations, JSDoc casts, and ignore.
Inactive directives may become active; do not freeze source to prevent that.
JavaScript directive prologues remain the exception above. Tool consumption alone
does not establish a layout contract.

## Array holes

Preserve hole counts and element indices. No comment may cross a hole or comma
boundary; whitespace and indentation may change. This intentionally differs from
Prettier. An unchanged ignore target does not excuse movement or instability.

<!-- prettier-ignore -->
```js
[1, /* note */, 2]; // Keep.
[1 /* note */, , 2]; // Do not move the comment here.
```

## Ignore and idempotence

Preserve effective `prettier-ignore` targets and protected raw contents.
With identical options, the first output must already be a byte-for-byte fixed point,
including line endings and the final newline:

```text
format(format(source)) == format(source)
```

Do not converge through repeated formatting or output postprocessing. Native/plugin
parity permits only verified, precisely bounded Unicode-width differences; each side
must independently pass fixed-point and consumer checks.

## Display width

The native formatter uses non-CJK `unicode-width` string widths, intentionally
allowing line breaks to differ from Prettier. JavaScript string-length rules remain
separate. Width accumulates per text instruction; splitting a Unicode sequence may
measure differently from its concatenation.

## Source and visual lines

Only CR, LF, and CRLF break visual lines. U+2028/U+2029 neither reset columns, split
comment text, count as visual newlines, nor independently expand groups.

Syntax, ASI, ignore ownership, and diagnostic coverage still use ECMAScript source
lines. Terminate line comments before following code. These layout rules do not
redefine the parser, AST, or Doc encoding.

## Implementation boundaries

Derive relationships from the complete original source and carry borrowed facts into
printing; cursor or output state must not change them. Parent syntax owns operators,
groups, and indentation. Separate raw-content ownership from source frames, and keep
language policy in the language formatter.

Avoid repeated parsing, overlapping scans, and unnecessary hot-path allocations.
Preserve generic Doc/IR contracts and real recursion guards. Express validation
constraints through syntax, ownership, and shared layout, not a production rule table.

## Intentional differences

Require a bounded reason and regression coverage. Check semantic and directive effects
with actual consumers and enabled/disabled-marker controls. Retain raw comparison
results; bind exceptions to exact cases and options. Revisit changed inputs, options,
outputs, or reference versions.

Do not sacrifice stable compatible layouts to fix another case's fixed point. Check
adjacent controls, source/AST parity, and first/second-pass output separately. Keep
unresolved results visible; AST checks do not prove runtime or consumer equivalence.
