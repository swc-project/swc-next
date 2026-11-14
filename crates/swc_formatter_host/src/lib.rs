//! Language-independent formatting sessions and document production.
//!
//! A [`FormatSession`] owns storage and shared layout options for one formatting
//! operation. Language producers retain their own parser configuration, syntax
//! options and errors. Produced and decoded documents can be composed with
//! [`Writer::write_doc`](swc_next_formatter_core::Writer::write_doc) before printing.
//! Applications supply an [`EmbeddedDispatcher`]; the host controls nested calls
//! through [`FormatContext`] without recognizing language syntax.

#![deny(clippy::all)]
#![deny(unused)]

mod context;
mod embedded;

use std::fmt;

use swc_next_allocator::Allocator;
use swc_next_formatter_core::{Doc, DocumentBufferError, PrintOptions};

pub use context::FormatContext;
pub use embedded::{
    EmbeddedDispatcher, EmbeddedError, EmbeddedErrorKind, EmbeddedOptions, EmbeddedOutcome,
    EmbeddedRequest, InputKind, PreserveReason,
};

/// A language-specific source-to-document implementation.
///
/// The output may borrow the source and session storage. It must remain usable
/// after the producer, its syntax options and the context's dispatcher are
/// dropped. Temporary parsed state must either live in the session or be
/// released after copying the document's borrowed data into session storage.
///
/// Each returned document has an independent named-group scope. Compose it
/// through [`Writer::import_doc`](swc_next_formatter_core::Writer::import_doc)
/// or [`Writer::write_doc`](swc_next_formatter_core::Writer::write_doc), which
/// remap group IDs. Label IDs remain caller-owned.
///
/// This contract operates on Rust documents; the serialized document protocol
/// remains the boundary between separately loaded runtime bindings.
pub trait DocumentProducer {
    /// Language-specific document construction options, excluding shared layout.
    type Options: ?Sized;
    /// Parsing or document construction failure reported by this producer.
    type Error;

    /// Produces a complete layout document using the current context.
    fn produce<'a>(
        &self,
        context: FormatContext<'a, '_>,
        source: &'a str,
        options: &Self::Options,
    ) -> Result<Doc<'a>, Self::Error>;
}

/// Storage and layout configuration shared by documents in one operation.
///
/// Documents borrow this session and their input text or encoded buffers. Print
/// or encode them before dropping those inputs or the session. The session does
/// not retain producer instances, load plugins, or infer a source language.
/// Debug output displays layout options without inspecting arena contents.
pub struct FormatSession {
    allocator: Allocator,
    print_options: PrintOptions,
}

impl FormatSession {
    pub fn new(print_options: PrintOptions) -> Self {
        Self {
            allocator: Allocator::new(),
            print_options,
        }
    }

    /// Returns the arena for document and producer storage.
    ///
    /// Arena allocations do not run destructors. Producers must release
    /// resources requiring `Drop` themselves rather than putting them here.
    pub fn allocator(&self) -> &Allocator {
        &self.allocator
    }

    /// Returns the shared layout options for document construction and printing.
    pub fn print_options(&self) -> PrintOptions {
        self.print_options
    }

    /// Creates a root context without an embedded-language dispatcher.
    pub fn context(&self) -> FormatContext<'_, 'static> {
        FormatContext::root(self, None, EmbeddedOptions::default())
    }

    /// Creates a root context with application-provided embedded dispatch.
    ///
    /// Documents borrow the session and their source, independently of the
    /// dispatcher lifetime. Nested producers must use the child context passed
    /// to the dispatcher to retain the recursion budget and shared services.
    pub fn context_with_embedded<'a, 'services>(
        &'a self,
        dispatcher: &'services dyn EmbeddedDispatcher,
        options: EmbeddedOptions,
    ) -> FormatContext<'a, 'services> {
        FormatContext::root(self, Some(dispatcher), options)
    }

    /// Produces a document without erasing language-specific options or errors.
    pub fn produce<'a, P: DocumentProducer + ?Sized>(
        &'a self,
        producer: &P,
        source: &'a str,
        options: &P::Options,
    ) -> Result<Doc<'a>, P::Error> {
        self.context().produce(producer, source, options)
    }

    /// Decodes a document, borrowing text from the supplied buffer.
    pub fn decode<'a>(&'a self, buffer: &'a [u8]) -> Result<Doc<'a>, DocumentBufferError> {
        Doc::decode(&self.allocator, buffer)
    }
}

impl Default for FormatSession {
    fn default() -> Self {
        Self::new(PrintOptions::default())
    }
}

impl fmt::Debug for FormatSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FormatSession")
            .field("print_options", &self.print_options)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
