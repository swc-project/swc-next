use std::fmt;

use swc_next_allocator::Allocator;
use swc_next_formatter_core::{Doc, PrintOptions};

use crate::{DocumentProducer, FormatSession};

/// Per-input view of a formatting session with its own layout configuration.
#[derive(Clone, Copy)]
pub struct FormatContext<'a> {
    session: &'a FormatSession,
    print_options: PrintOptions,
}

impl<'a> FormatContext<'a> {
    pub(crate) fn root(session: &'a FormatSession) -> Self {
        Self {
            session,
            print_options: session.print_options(),
        }
    }

    /// Returns the shared arena. Allocations do not run destructors.
    pub fn allocator(self) -> &'a Allocator {
        self.session.allocator()
    }

    /// Returns layout options for this input.
    pub fn print_options(self) -> PrintOptions {
        self.print_options
    }

    /// Changes layout for this context. Other contexts are unaffected.
    #[must_use]
    pub fn with_print_options(self, print_options: PrintOptions) -> Self {
        Self {
            print_options,
            ..self
        }
    }

    /// Produces a document using the current context.
    pub fn produce<P: DocumentProducer + ?Sized>(
        self,
        producer: &P,
        source: &'a str,
        options: &P::Options,
    ) -> Result<Doc<'a>, P::Error> {
        producer.produce(self, source, options)
    }
}

impl fmt::Debug for FormatContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FormatContext")
            .field("print_options", &self.print_options)
            .finish_non_exhaustive()
    }
}
