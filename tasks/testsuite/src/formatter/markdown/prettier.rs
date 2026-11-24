use std::{
    cell::Cell,
    collections::{HashMap, hash_map::Entry},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serialize, de};
use swc_next_allocator::CloneIn;
use swc_next_formatter::markdown::{MarkdownDocumentProducer, SyntaxOptions};
use swc_next_formatter_core::Doc;
use swc_next_formatter_host::{
    EmbeddedDispatcher, EmbeddedError, EmbeddedOutcome, EmbeddedRequest, FormatContext,
    PreserveReason,
};

use super::OptionSet;

#[derive(Serialize)]
struct Request<'a> {
    source: &'a str,
    options: &'a OptionSet,
    filepath: &'a Path,
}

#[derive(Deserialize)]
struct Response {
    fences: Vec<Fence>,
}

#[derive(Deserialize)]
struct Fence {
    #[serde(default)]
    parent: usize,
    language: String,
    source: String,
    #[serde(default)]
    markdown: bool,
    #[serde(deserialize_with = "decode_doc")]
    doc: Option<Vec<u8>>,
}

fn decode_doc<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|doc| STANDARD.decode(doc).map_err(de::Error::custom))
        .transpose()
}

pub(super) struct Collector {
    child: Child,
    input: Option<BufWriter<ChildStdin>>,
    output: BufReader<ChildStdout>,
    cache: HashMap<Vec<u8>, Vec<Fence>>,
}

impl Collector {
    pub(super) fn start(crate_root: &Path) -> Result<Self, String> {
        let script = crate_root.join("scripts/prettier-markdown-fences.mjs");
        let mut child = Command::new("node")
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| format!("start {}: {error}", script.display()))?;
        let input = BufWriter::new(child.stdin.take().expect("piped stdin"));
        let output = BufReader::new(child.stdout.take().expect("piped stdout"));
        Ok(Self {
            child,
            input: Some(input),
            output,
            cache: HashMap::new(),
        })
    }

    pub(super) fn finish(mut self) -> Result<(), String> {
        drop(self.input.take());
        let mut tail = String::new();
        if self.output.read_line(&mut tail).map_err(output_error)? != 0 {
            return Err("unexpected extra response from Prettier fence collector".into());
        }
        let status = self.child.wait().map_err(process_error)?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Prettier fence collector exited with {status}"))
        }
    }

    pub(super) fn format(
        &mut self,
        source: &str,
        options: &OptionSet,
        filepath: &Path,
        syntax: SyntaxOptions,
        render: impl Fn(&dyn EmbeddedDispatcher) -> Result<String, EmbeddedError>,
    ) -> Result<Rendered, String> {
        let fences = self.fences(source, options, filepath)?;
        render_fences(fences, syntax, render)
    }

    fn fences(
        &mut self,
        source: &str,
        options: &OptionSet,
        filepath: &Path,
    ) -> Result<&[Fence], String> {
        let request = serde_json::to_vec(&Request {
            source,
            options,
            filepath,
        })
        .map_err(|error| format!("encode Prettier fence request: {error}"))?;
        Ok(match self.cache.entry(request) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let stdin = self.input.as_mut().expect("collector is running");
                stdin
                    .write_all(entry.key())
                    .and_then(|()| stdin.write_all(b"\n"))
                    .and_then(|()| stdin.flush())
                    .map_err(|error| format!("send Prettier fence request: {error}"))?;
                let mut line = String::new();
                if self.output.read_line(&mut line).map_err(output_error)? == 0 {
                    let status = self.child.try_wait().map_err(process_error)?;
                    let detail = status.map_or_else(String::new, |status| format!(" ({status})"));
                    return Err(format!(
                        "Prettier fence collector closed stdout before responding{detail}"
                    ));
                }
                let response: Response = serde_json::from_str(&line)
                    .map_err(|error| format!("decode Prettier fence response: {error}"))?;
                entry.insert(response.fences)
            }
        })
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

pub(super) struct Rendered {
    pub(super) text: String,
    pub(super) alignment: Option<String>,
}

fn render_fences(
    fences: &[Fence],
    syntax: SyntaxOptions,
    render: impl Fn(&dyn EmbeddedDispatcher) -> Result<String, EmbeddedError>,
) -> Result<Rendered, String> {
    let dispatcher = Dispatcher::new(fences, syntax, true)?;
    let mut text = render(&dispatcher).map_err(|error| error.to_string())?;
    let alignment = dispatcher.alignment_error();
    if alignment.is_some() {
        let preserve = Dispatcher::new(fences, syntax, false)?;
        text = render(&preserve).map_err(|error| error.to_string())?;
    }
    Ok(Rendered { text, alignment })
}

#[derive(Default)]
struct FenceGroup<'a> {
    occurrences: Vec<(usize, &'a Fence)>,
    visited: Cell<usize>,
    has_documents: bool,
}

struct Dispatcher<'a> {
    groups: HashMap<(usize, &'a str, &'a str), FenceGroup<'a>>,
    parent: Cell<Option<usize>>,
    syntax: SyntaxOptions,
    use_documents: bool,
}

impl<'a> Dispatcher<'a> {
    fn new(
        fences: &'a [Fence],
        syntax: SyntaxOptions,
        use_documents: bool,
    ) -> Result<Self, String> {
        let mut groups: HashMap<_, FenceGroup<'a>> = HashMap::new();
        let mut descendant_documents = vec![false; fences.len() + 1];
        // Mark the Markdown ancestors that establish a path to an official Doc.
        for (index, fence) in fences.iter().enumerate().rev() {
            let ordinal = index + 1;
            if fence.parent >= ordinal || (fence.parent != 0 && !fences[fence.parent - 1].markdown)
            {
                return Err(format!(
                    "invalid parent {} for official fence #{ordinal}",
                    fence.parent,
                ));
            }
            if fence.markdown && fence.doc.is_some() {
                return Err(format!(
                    "unexpected guest document for Markdown fence #{ordinal}",
                ));
            }

            let has_documents = fence.doc.is_some() || descendant_documents[ordinal];
            descendant_documents[fence.parent] |= has_documents;
            let key = (fence.parent, fence.language.as_str(), fence.source.as_str());
            let group = groups.entry(key).or_default();
            group.occurrences.push((ordinal, fence));
            group.has_documents |= has_documents;
        }
        for group in groups.values_mut() {
            group.occurrences.reverse();
        }
        Ok(Self {
            groups,
            parent: Cell::new(Some(0)),
            syntax,
            use_documents,
        })
    }

    fn alignment_error(&self) -> Option<String> {
        let group = self
            .groups
            .values()
            .filter(|group| group.has_documents && group.visited.get() != group.occurrences.len())
            .min_by_key(|group| group.occurrences[0].0)?;
        let (ordinal, fence) = group.occurrences[0];
        Some(format!(
            "fence group starting at official #{} under parent #{} {:?} ({} body bytes): {} official occurrences, {} native requests",
            ordinal,
            fence.parent,
            fence.language,
            fence.source.len(),
            group.occurrences.len(),
            group.visited.get(),
        ))
    }

    fn next_fence(&self, language: &str, source: &str) -> Option<(usize, &Fence)> {
        let parent = self.parent.get()?;
        let group = self.groups.get(&(parent, language, source))?;
        let index = group.visited.get();
        group.visited.set(index + 1);
        group.occurrences.get(index).copied()
    }

    fn format_markdown<'session>(
        &self,
        context: FormatContext<'session, '_>,
        source: &'session str,
        parent: Option<usize>,
    ) -> Result<EmbeddedOutcome<'session>, EmbeddedError> {
        let previous = self.parent.replace(parent);
        let document = context.produce(&MarkdownDocumentProducer, source, &self.syntax);
        self.parent.set(previous);
        document.map(EmbeddedOutcome::Formatted)
    }
}

impl EmbeddedDispatcher for Dispatcher<'_> {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        let preserve = Ok(EmbeddedOutcome::PreserveOriginal(
            PreserveReason::Unsupported,
        ));
        let Some((ordinal, fence)) = self.next_fence(request.language, request.source) else {
            if request.language.eq_ignore_ascii_case("md")
                || request.language.eq_ignore_ascii_case("markdown")
            {
                return self.format_markdown(context, request.source, None);
            }
            return preserve;
        };
        if fence.markdown {
            return self.format_markdown(context, request.source, Some(ordinal));
        }
        let Some(bytes) = fence.doc.as_ref().filter(|_| self.use_documents) else {
            return preserve;
        };
        let allocator = context.allocator();
        let bytes = bytes.as_slice().clone_in(allocator);
        let document = Doc::decode(allocator, bytes).map_err(|error| {
            EmbeddedError::new(io::Error::other(format!(
                "decode Prettier document for fence #{} {:?}: {error}",
                ordinal, fence.language,
            )))
        })?;
        Ok(EmbeddedOutcome::Formatted(document))
    }
}

fn output_error(error: io::Error) -> String {
    format!("read Prettier fence response: {error}")
}

fn process_error(error: io::Error) -> String {
    format!("wait for Prettier fence collector: {error}")
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_formatter::markdown;
    use swc_next_formatter_core::Writer;

    use super::*;

    fn fence(language: &str, source: &str, doc: Option<Vec<u8>>) -> Fence {
        Fence {
            parent: 0,
            language: language.into(),
            source: source.into(),
            markdown: false,
            doc,
        }
    }

    fn formatted(text: &str) -> Vec<u8> {
        let arena = Allocator::new();
        let mut writer = Writer::new(&arena);
        writer.text(text);
        writer.finish().encode()
    }

    fn render(source: &str, fences: &[Fence]) -> Result<Rendered, String> {
        render_fences(fences, SyntaxOptions::default(), |dispatcher| {
            markdown::format_with_dispatcher(source, markdown::FormatOptions::default(), dispatcher)
        })
    }

    #[test]
    fn explicit_fallback() {
        let response = r#"{"fences":[{"language":"js","source":"one","doc":null}]}"#;
        assert!(
            serde_json::from_str::<Response>(response).unwrap().fences[0]
                .doc
                .is_none()
        );

        let response = r#"{"fences":[{"language":"js","source":"one"}]}"#;
        assert!(serde_json::from_str::<Response>(response).is_err());

        let response = r#"{"fences":[{"language":"js","source":"one","doc":"!"}]}"#;
        assert!(serde_json::from_str::<Response>(response).is_err());

        let invalid = [fence("js", "one", Some(vec![0]))];
        assert!(render("```js\none\n```\n", &invalid).is_err());
    }

    #[test]
    fn fence_alignment() {
        let source = "```js\none\n```\n\n```ts\ntwo\n```\n";
        let matching = [
            fence("js", "one", Some(formatted("formatted"))),
            fence("ts", "two", None),
        ];
        let escaped_label = "```j&#115;\none\n```\n\n```ts\ntwo\n```\n";
        let result = render(escaped_label, &matching).unwrap();
        assert_eq!(result.text, "```js\nformatted\n```\n\n```ts\ntwo\n```\n");
        assert!(result.alignment.is_none());

        let different_body = [
            fence("js", "one", Some(formatted("formatted"))),
            fence("ts", "TWO", Some(formatted("formatted"))),
        ];
        let result = render(source, &different_body).unwrap();
        assert_eq!(result.text, source);
        assert!(result.alignment.is_some());

        let ambiguous = [
            fence("js", "one", None),
            fence("js", "one", Some(formatted("formatted"))),
        ];
        let result = render(source, &ambiguous).unwrap();
        assert_eq!(result.text, source);
        assert!(result.alignment.is_some());

        let result = render("```js\none\n```\n\n```js\none\n```\n", &ambiguous).unwrap();
        assert_eq!(result.text, "```js\none\n```\n\n```js\nformatted\n```\n");
        assert!(result.alignment.is_none());

        let different_fallback = [
            fence("rs", "unseen", None),
            fence("ts", "two", Some(formatted("formatted"))),
        ];
        let result = render(source, &different_fallback).unwrap();
        assert_eq!(result.text, "```js\none\n```\n\n```ts\nformatted\n```\n");
        assert!(result.alignment.is_none());
    }

    #[test]
    fn preservation_without_guest_documents() {
        let fences = [
            fence("rs", "one", None),
            fence("rs", "two", None),
            fence("rs", "three", None),
        ];
        let result = render("# Heading\n", &fences).unwrap();
        assert_eq!(result.text, "# Heading\n");
        assert!(result.alignment.is_none());
    }
}
