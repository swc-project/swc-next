//! Inspect concrete target nodes and the source tokens on a diagnostic line.

use std::collections::HashMap;

use anyhow::{Result, bail};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Comment, CommentKind, GetSpan, NodeId, NodeKind, NodeKindData, Span, Visit, VisitWith,
};
use swc_next_ecma_parser::{Options, Parser, Token, TokenParserConfig, TokenTag};

use super::{Marker, Occurrence, Snapshot, Target};

// Positions in this traversal distinguish two identical calls in different
// parts of a file. Parenthesis wrappers and empty statements may disappear.
struct Syntax<'a, 'source> {
    ast: &'a Ast<'source>,
    shape: Vec<(NodeKind, usize)>,
    spans: Vec<Span>,
    nodes: HashMap<NodeId, usize>,
    calls: HashMap<u32, Vec<usize>>,
    argument_gaps: HashMap<u32, u32>,
    pairs: HashMap<u32, usize>,
    strings: HashMap<u32, &'source [u8]>,
    prologues: Vec<(usize, Vec<String>)>,
}

impl<'source> Visit<'source> for Syntax<'_, 'source> {
    fn ast(&self) -> &Ast<'source> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        let ast = self.ast;
        let id = node.node_id();
        let kind = ast.node_kind(id);
        let start = ast.span(id).start;
        let directives = match node {
            NodeKindData::Program(program) => Some(program.directives(ast)),
            NodeKindData::FunctionBody(body) => Some(body.directives(ast)),
            _ => None,
        };
        if let Some(directives) = directives {
            let values = directives
                .iter()
                .map(|id| {
                    let directive = ast.get_node_in_sub_range(id);
                    ast.get_utf8(directive.value(ast)).to_owned()
                })
                .collect();
            self.prologues.push((self.shape.len(), values));
        }
        match node {
            NodeKindData::EmptyStatement(_) | NodeKindData::ParenthesizedExpression(_) => return,
            NodeKindData::CallExpression(_) | NodeKindData::NewExpression(_) => {
                self.calls.entry(start).or_default().push(self.shape.len());
                let arguments = match node {
                    NodeKindData::CallExpression(call) => call.arguments(ast),
                    NodeKindData::NewExpression(call) => call.arguments(ast),
                    _ => unreachable!(),
                };
                let mut previous = None;
                for id in arguments.iter() {
                    let span = ast.get_node_in_sub_range(id).span(ast);
                    if let Some(end) = previous {
                        self.argument_gaps.insert(end, span.start);
                    }
                    previous = Some(span.end);
                }
            }
            NodeKindData::StringLiteral(string) => {
                self.strings
                    .insert(start, ast.get_wtf8(string.value(ast)).as_bytes());
            }
            _ => {}
        }
        self.nodes.insert(id, self.shape.len());
        // Keep a string statement aligned even when a removed barrier changes its
        // directive status. The scope's original directive list checks that status.
        let kind = if kind == NodeKind::Directive {
            NodeKind::ExpressionStatement
        } else {
            kind
        };
        self.shape.push((kind, 0));
        self.spans.push(ast.span(id));
    }

    fn leave_node(&mut self, node: NodeKindData) {
        if let NodeKindData::ParenthesizedExpression(pair) = node {
            let inner = self.nodes[&pair.expression(self.ast).node_id()];
            self.pairs
                .insert(self.ast.span(pair.node_id()).start, inner);
            self.nodes.insert(pair.node_id(), inner);
        } else if let Some(&index) = self.nodes.get(&node.node_id()) {
            // A preorder list alone cannot distinguish f(g(x), y) from f(g(x, y)).
            self.shape[index].1 = self.shape.len();
        }
    }
}

fn classify(raw: &str, kind: CommentKind) -> Option<(Marker, &str)> {
    let value = match kind {
        CommentKind::Line => raw.strip_prefix("//")?,
        CommentKind::Block => raw.strip_prefix("/*")?.strip_suffix("*/")?,
    }
    .trim();
    let pure = value.contains("@__PURE__") || value.contains("#__PURE__");
    let cast = raw.starts_with("/**")
        && value
            .match_indices("@type")
            .any(|(i, _)| value[i + 5..].starts_with(|c: char| c.is_whitespace() || c == '{'));
    let diagnostic = value.contains("@ts-ignore") || value.contains("@ts-expect-error");
    match (pure, cast, diagnostic) {
        (false, false, false) => None,
        (true, false, false)
            if kind == CommentKind::Block && matches!(value, "#__PURE__" | "@__PURE__") =>
        {
            Some((Marker::Pure, value))
        }
        (false, true, false) => {
            let body = value.strip_prefix('*')?.trim();
            let tag = body.strip_prefix("@type").map(str::trim);
            let simple = tag.is_some_and(|tag| {
                tag.starts_with('{')
                    && tag.ends_with('}')
                    && !tag.contains(['\n', '\r', '\u{2028}', '\u{2029}', '@'])
            }) && !raw.contains(['\n', '\r', '\u{2028}', '\u{2029}']);
            Some((
                if simple {
                    Marker::Cast
                } else {
                    Marker::Unsupported
                },
                value,
            ))
        }
        (false, false, true) => {
            let last = raw
                .rsplit(['\n', '\r', '\u{2028}', '\u{2029}'])
                .next()?
                .trim_start();
            let tag = if kind == CommentKind::Block {
                last.trim_start_matches(['/', '*']).trim_start()
            } else {
                let body = last.strip_prefix("//")?;
                body.strip_prefix('/').unwrap_or(body).trim_start()
            };
            let marker = if value.matches("@ts-").count() != 1 {
                Marker::Unsupported
            } else if tag.starts_with("@ts-expect-error") {
                Marker::ExpectError
            } else if tag.starts_with("@ts-ignore") {
                Marker::Ignore
            } else {
                Marker::Unsupported
            };
            Some((marker, value))
        }
        _ => Some((Marker::Unsupported, value)),
    }
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    let mut chars = source.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '\r' if chars.next_if(|(_, c)| *c == '\n').is_some() => starts.push(at + 2),
            '\r' | '\n' | '\u{2028}' | '\u{2029}' => starts.push(at + c.len_utf8()),
            _ => {}
        }
    }
    starts
}

struct Tokens<'a> {
    source: &'a str,
    all: &'a [Token],
    code: Vec<Token>,
    lines: Vec<usize>,
    diagnostic_counts: Vec<usize>,
    punctuation: Vec<Option<(usize, usize)>>,
}

impl Tokens<'_> {
    fn line(&self, offset: u32) -> usize {
        self.lines
            .partition_point(|&start| start <= offset as usize)
            - 1
    }

    fn next(&self, offset: u32) -> Option<Token> {
        self.all
            .get(self.all.partition_point(|t| t.start() < offset))
            .copied()
            .filter(|t| t.tag() != TokenTag::Eof)
    }

    fn diagnostic_target(&self, directive_line: usize) -> (Target, Option<Span>) {
        if self.diagnostic_counts[directive_line] != 1 {
            return (
                Target::Unknown("competing diagnostic directives on one line"),
                None,
            );
        }
        for line in directive_line + 1..self.lines.len() {
            let start = self.lines[line];
            let end = self
                .lines
                .get(line + 1)
                .copied()
                .unwrap_or(self.source.len());
            let text = self.source[start..end].trim();
            if text.is_empty() || text.starts_with("//") {
                if self.diagnostic_counts[line] > 0 {
                    return (Target::Inactive, None);
                }
                continue;
            }
            let first = self.code.partition_point(|t| (t.start() as usize) < start);
            let last = self.code.partition_point(|t| (t.start() as usize) < end);
            let all_first = self.all.partition_point(|t| (t.end() as usize) <= start);
            let all_end = self.all.partition_point(|t| (t.start() as usize) < end);
            let all = &self.all[all_first..all_end];
            if all.is_empty() {
                // Blocks of comment text stop the search for a diagnostic on a later code line.
                return (Target::Inactive, None);
            }
            if (all[0].start() as usize) < start || all.last().unwrap().end() as usize > end {
                return (
                    Target::Unknown("diagnostic line intersects a multiline token"),
                    None,
                );
            }
            let punctuation = self.punctuation[all_first..all_end]
                .iter()
                .flatten()
                .copied()
                .collect();
            return (
                Target::Line {
                    first,
                    end: last,
                    punctuation,
                },
                Some(Span::new(all[0].start(), all.last().unwrap().end())),
            );
        }
        (Target::Inactive, None)
    }
}

fn attachment(
    marker: &Marker,
    comment: Comment,
    tokens: &Tokens<'_>,
    syntax: &Syntax<'_, '_>,
) -> Target {
    if *marker == Marker::Unsupported {
        return Target::Unknown("unrecognized or mixed directive form");
    }
    if *marker == Marker::Cast && syntax.ast.is_ts() {
        return Target::Unknown("JSDoc casts are checked in JS and JSX");
    }
    let Some(next) = tokens.next(comment.span.end) else {
        return Target::Inactive;
    };
    if !tokens.source[comment.span.end as usize..next.start() as usize]
        .trim()
        .is_empty()
    {
        return Target::Unknown("another comment intervenes before the target");
    }
    if next.tag() == TokenTag::Semicolon {
        return Target::Inactive;
    }
    if *marker == Marker::Pure && next.tag() == TokenTag::Comma {
        let before = tokens
            .all
            .partition_point(|t| t.end() <= comment.span.start);
        if let Some(previous) = before.checked_sub(1).map(|i| tokens.all[i])
            && syntax
                .argument_gaps
                .get(&previous.end())
                .is_some_and(|&end| next.end() <= end)
        {
            // A marker before the argument separator cannot annotate the next argument.
            return Target::Inactive;
        }
    }
    let node = match marker {
        Marker::Pure => match syntax.calls.get(&next.start()).map(Vec::as_slice) {
            Some([only]) => *only,
            Some(_) => return Target::Unknown("multiple calls start at this token"),
            None if next.tag() == TokenTag::Identifier
                && tokens
                    .next(next.end())
                    .is_none_or(|following| following.tag() == TokenTag::Semicolon) =>
            {
                return Target::Inactive;
            }
            None => return Target::Unknown("no direct call or new expression at this token"),
        },
        Marker::Cast => {
            if next.tag() != TokenTag::LeftParen {
                // Other JSDoc annotations can describe declarations without forming a cast.
                return Target::Inactive;
            }
            let Some(inner) = syntax.pairs.get(&next.start()) else {
                return Target::Unknown("no explicit cast parentheses at this token");
            };
            *inner
        }
        _ => unreachable!(),
    };
    Target::Node(node, syntax.shape[node].0)
}

pub(super) fn inspect(source: &str, options: Options) -> Result<Snapshot> {
    let arena = Allocator::new();
    let parsed = Parser::init(&arena, source, options, TokenParserConfig).parse();
    if let Some(error) = parsed.diagnostics.first() {
        bail!("SWC parse error at {:?}: {}", error.span, error.message);
    }
    // SAFETY: All handles belong to this read-only parse for the duration of the traversal.
    let ast = unsafe { parsed.ast.ast() };
    let mut syntax = Syntax {
        ast,
        shape: Vec::new(),
        spans: Vec::new(),
        nodes: HashMap::new(),
        calls: HashMap::new(),
        argument_gaps: HashMap::new(),
        pairs: HashMap::new(),
        strings: HashMap::new(),
        prologues: Vec::new(),
    };
    ast.root_program().visit_with(&mut syntax);
    let mut code = Vec::new();
    let mut gaps = vec![Vec::new()];
    let mut punctuation = Vec::new();
    let all = &parsed.tokens[..parsed.tokens.partition_point(|t| t.tag() != TokenTag::Eof)];
    for &t in all {
        if matches!(
            t.tag(),
            TokenTag::LeftParen | TokenTag::RightParen | TokenTag::Semicolon | TokenTag::Comma
        ) {
            let gap = gaps.last_mut().unwrap();
            punctuation.push(Some((code.len(), gap.len())));
            gap.push(t.tag());
        } else {
            punctuation.push(None);
            code.push(t);
            gaps.push(Vec::new());
        }
    }
    let lines = line_starts(source);
    let mut tokens = Tokens {
        source,
        all,
        code,
        diagnostic_counts: vec![0; lines.len()],
        lines,
        punctuation,
    };
    let mut markers = Vec::new();
    for &c in ast.comments() {
        let raw = &source[c.span.start as usize..c.span.end as usize];
        if let Some((marker, text)) = classify(raw, c.kind) {
            if matches!(marker, Marker::Ignore | Marker::ExpectError) {
                let line = tokens.line(c.span.end);
                tokens.diagnostic_counts[line] += 1;
            }
            markers.push((c, marker, text));
        }
    }
    let directives = markers
        .into_iter()
        .map(|(c, marker, text)| {
            let (target, target_span) = if matches!(marker, Marker::Ignore | Marker::ExpectError) {
                tokens.diagnostic_target(tokens.line(c.span.end))
            } else {
                let target = attachment(&marker, c, &tokens, &syntax);
                let span = match target {
                    Target::Node(i, _) => Some(syntax.spans[i]),
                    _ => None,
                };
                (target, span)
            };
            Occurrence {
                marker,
                text: text.to_owned(),
                line: 1 + tokens.line(c.span.start),
                target,
                target_span,
            }
        })
        .collect();
    let code = tokens
        .code
        .iter()
        .map(|t| {
            let raw = &source.as_bytes()[t.start() as usize..t.end() as usize];
            (
                t.tag(),
                syntax
                    .strings
                    .get(&t.start())
                    .copied()
                    .unwrap_or(raw)
                    .to_vec(),
            )
        })
        .collect();
    Ok(Snapshot {
        shape: syntax.shape,
        gaps,
        code,
        directives,
        prologues: syntax.prologues,
    })
}
