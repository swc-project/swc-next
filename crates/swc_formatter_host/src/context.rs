use std::{borrow::Cow, fmt};

use swc_next_allocator::Allocator;
use swc_next_formatter_core::{Doc, PrintOptions};

use crate::{
    DocumentProducer, EmbeddedDispatcher, EmbeddedError, EmbeddedOptions, EmbeddedOutcome,
    EmbeddedRequest, FormatSession, InputKind, PreserveReason,
};

/// Per-input view of a formatting session and its embedded-language services.
///
/// The storage lifetime is independent of the borrowed dispatcher lifetime.
/// Copies share the arena and services, with their own layout and depth values.
/// Only [`Self::embed`] derives a child; producing a document on that child
/// retains its depth instead of starting another root operation.
/// Debug output displays effective configuration and depth, omitting arena and
/// dispatcher internals.
#[derive(Clone, Copy)]
pub struct FormatContext<'a, 'services> {
    session: &'a FormatSession,
    dispatcher: Option<&'services dyn EmbeddedDispatcher>,
    embedded_options: EmbeddedOptions,
    print_options: PrintOptions,
    input_kind: InputKind,
    depth: u8,
}

impl<'a, 'services> FormatContext<'a, 'services> {
    pub(crate) fn root(
        session: &'a FormatSession,
        dispatcher: Option<&'services dyn EmbeddedDispatcher>,
        embedded_options: EmbeddedOptions,
    ) -> Self {
        Self {
            session,
            dispatcher,
            embedded_options,
            print_options: session.print_options(),
            input_kind: InputKind::Document,
            depth: 0,
        }
    }

    /// Returns the shared arena. Allocations do not run destructors.
    pub fn allocator(self) -> &'a Allocator {
        self.session.allocator()
    }

    /// Returns layout options inherited by this input and its children.
    pub fn print_options(self) -> PrintOptions {
        self.print_options
    }

    /// Changes layout for this context and its descendants, retaining depth
    /// and services. Other contexts from the same session are unaffected.
    #[must_use]
    pub fn with_print_options(self, print_options: PrintOptions) -> Self {
        Self {
            print_options,
            ..self
        }
    }

    /// Returns this input's envelope. Roots are complete documents; children
    /// carry the kind requested by their parent. This does not select a parser.
    pub fn input_kind(self) -> InputKind {
        self.input_kind
    }

    /// Root inputs have depth zero; the first embedded input has depth one.
    pub fn depth(self) -> u8 {
        self.depth
    }

    /// Produces a document without resetting the current embedding context.
    pub fn produce<P: DocumentProducer + ?Sized>(
        self,
        producer: &P,
        source: &'a str,
        options: &P::Options,
    ) -> Result<Doc<'a>, P::Error> {
        producer.produce(self, source, options)
    }

    /// Dispatches one embedded input on a derived child context.
    ///
    /// Borrowed source remains borrowed. Owned source is copied into the arena
    /// once, allowing a producer's temporary transformed input to outlive the
    /// call. Language identifiers are only borrowed for the dispatch call.
    ///
    /// Disabled dispatch and missing dispatchers preserve the original input
    /// without invoking a provider or copying source. Providers decide which
    /// languages and inputs they support. Preservation returns no substitute
    /// Doc: the parent retains its original syntax, escaping and whitespace.
    /// Formatted Docs are not trimmed, printed or otherwise rewritten here.
    ///
    /// # Errors
    ///
    /// Returns a depth-limit error before invoking a provider beyond the budget.
    /// Provider errors propagate unchanged; they are never converted to fallback.
    pub fn embed(
        self,
        language: &str,
        source: impl Into<Cow<'a, str>>,
        input_kind: InputKind,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
        if !self.embedded_options.enabled {
            return Ok(EmbeddedOutcome::PreserveOriginal(PreserveReason::Disabled));
        }
        let Some(dispatcher) = self.dispatcher else {
            return Ok(EmbeddedOutcome::PreserveOriginal(
                PreserveReason::NoDispatcher,
            ));
        };
        if self.depth >= self.embedded_options.max_depth {
            return Err(EmbeddedError::depth_limit(self.embedded_options.max_depth));
        }
        let source = match source.into() {
            Cow::Borrowed(source) => source,
            Cow::Owned(source) => self.allocator().alloc_str(&source),
        };
        let child = Self {
            input_kind,
            depth: self.depth + 1,
            ..self
        };
        dispatcher.dispatch(child, EmbeddedRequest { language, source })
    }
}

impl fmt::Debug for FormatContext<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FormatContext")
            .field("print_options", &self.print_options)
            .field("embedded_options", &self.embedded_options)
            .field("input_kind", &self.input_kind)
            .field("depth", &self.depth)
            .field("has_dispatcher", &self.dispatcher.is_some())
            .finish_non_exhaustive()
    }
}
