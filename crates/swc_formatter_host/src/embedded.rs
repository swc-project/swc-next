use std::{error::Error, fmt};

use swc_next_formatter_core::Doc;

use crate::FormatContext;

/// The envelope a parent asks a child producer to format.
///
/// This does not select a parser dialect or grant file-level ownership of a
/// BOM. Producers decide which envelopes they accept; the parent owns quoting,
/// interpolation, indentation wrappers and terminal-newline handling.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputKind {
    /// A complete language document, such as the contents of a code fence.
    Document,
    /// Content within another language's syntax, such as a template body.
    Fragment,
}

/// Policy inherited by all embedded calls in one root context.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EmbeddedOptions {
    /// Whether a configured dispatcher may be invoked.
    pub enabled: bool,
    /// Maximum nested dispatch depth, excluding the root producer.
    /// Zero rejects the first enabled dispatch; siblings do not share a counter.
    pub max_depth: u8,
}

impl Default for EmbeddedOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            max_depth: 8,
        }
    }
}

/// An application-selected language and source for one embedded call.
///
/// Source is valid for the session borrow; the language identifier need only
/// survive this call. The child context carries the requested [`InputKind`].
/// Debug output includes the language and source byte length, omitting source text.
pub struct EmbeddedRequest<'a, 'request> {
    /// Opaque to the host. The application maps names and aliases to producers.
    pub language: &'request str,
    pub source: &'a str,
}

impl fmt::Debug for EmbeddedRequest<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EmbeddedRequest")
            .field("language", &self.language)
            .field("source_len", &self.source.len())
            .finish_non_exhaustive()
    }
}

/// A deliberate decision to retain the parent's original source representation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PreserveReason {
    /// Embedded formatting is disabled for this operation.
    Disabled,
    /// The application did not install an embedded-language dispatcher.
    NoDispatcher,
    /// The provider declines the language, envelope or input. This includes
    /// child parse failures when its language policy chooses preservation.
    Unsupported,
}

/// Successful dispatch, including deliberate preservation of the original input.
///
/// `PreserveOriginal` is not a Doc containing reformatted source. The parent
/// keeps the syntax it would have emitted without embedding. A formatted empty
/// document remains a successful replacement and must not trigger fallback.
/// Debug output displays the outcome and preservation reason without traversing Docs.
pub enum EmbeddedOutcome<'a> {
    Formatted(Doc<'a>),
    PreserveOriginal(PreserveReason),
}

impl fmt::Debug for EmbeddedOutcome<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Formatted(_) => formatter.debug_tuple("Formatted").finish_non_exhaustive(),
            Self::PreserveOriginal(reason) => formatter
                .debug_tuple("PreserveOriginal")
                .field(reason)
                .finish(),
        }
    }
}

/// Application-provided routing between language producers.
///
/// The host supplies a child context with shared storage and services, inherited
/// layout and one additional depth level. Invoke child producers through that
/// context, and use its [`FormatContext::embed`] for further embedded calls.
/// Returning Docs permits structural composition before the final print; use
/// [`Writer::import_doc`](swc_next_formatter_core::Writer::import_doc) or
/// [`Writer::write_doc`](swc_next_formatter_core::Writer::write_doc) to remap their
/// independent named-group scopes. No Rust Doc crosses a runtime-binding boundary.
///
/// Language recognition, alias resolution, parser selection, syntax options and
/// expected source failures belong to this application/language boundary. Return
/// `PreserveOriginal` for deliberate refusal and `Err` for execution failures.
/// The host does not catch panics or turn errors into successful preservation.
///
/// ```
/// use swc_next_formatter_core::Writer;
/// use swc_next_formatter_host::{
///     EmbeddedDispatcher, EmbeddedError, EmbeddedOptions, EmbeddedOutcome,
///     EmbeddedRequest, FormatContext, FormatSession, InputKind, PreserveReason,
/// };
///
/// struct TextDispatcher;
/// impl EmbeddedDispatcher for TextDispatcher {
///     fn dispatch<'a>(
///         &self,
///         context: FormatContext<'a, '_>,
///         request: EmbeddedRequest<'a, '_>,
///     ) -> Result<EmbeddedOutcome<'a>, EmbeddedError> {
///         if request.language != "text" {
///             return Ok(EmbeddedOutcome::PreserveOriginal(PreserveReason::Unsupported));
///         }
///         let mut writer = Writer::new(context.allocator());
///         writer.literal_text(request.source);
///         Ok(EmbeddedOutcome::Formatted(writer.finish()))
///     }
/// }
///
/// let session = FormatSession::default();
/// let dispatcher = TextDispatcher;
/// let context = session.context_with_embedded(&dispatcher, EmbeddedOptions::default());
/// let outcome = context.embed("text", String::from("embedded λ"), InputKind::Fragment)?;
/// if let EmbeddedOutcome::Formatted(document) = outcome {
///     assert_eq!(document.print(context.print_options()), "embedded λ");
/// }
/// # Ok::<(), EmbeddedError>(())
/// ```
pub trait EmbeddedDispatcher {
    fn dispatch<'a>(
        &self,
        context: FormatContext<'a, '_>,
        request: EmbeddedRequest<'a, '_>,
    ) -> Result<EmbeddedOutcome<'a>, EmbeddedError>;
}

/// An operational failure, separate from deliberate preservation.
#[derive(Debug)]
pub struct EmbeddedError {
    cause: ErrorCause,
}

#[derive(Debug)]
enum ErrorCause {
    DepthLimit { max_depth: u8 },
    Provider(Box<dyn Error + Send + Sync>),
}

/// Classification of an embedded execution failure.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EmbeddedErrorKind {
    /// The next dispatch would exceed the root context's recursion budget.
    DepthLimit { max_depth: u8 },
    /// An application or language service failed; inspect the error source.
    Provider,
}

impl EmbeddedError {
    /// Creates a provider failure from a concrete error, retaining it as the source.
    /// Use `EmbeddedError::from` to take ownership of an already boxed error.
    pub fn new(source: impl Error + Send + Sync + 'static) -> Self {
        let source: Box<dyn Error + Send + Sync> = Box::new(source);
        Self::from(source)
    }

    pub fn kind(&self) -> EmbeddedErrorKind {
        match self.cause {
            ErrorCause::DepthLimit { max_depth } => EmbeddedErrorKind::DepthLimit { max_depth },
            ErrorCause::Provider(_) => EmbeddedErrorKind::Provider,
        }
    }

    pub(crate) fn depth_limit(max_depth: u8) -> Self {
        Self {
            cause: ErrorCause::DepthLimit { max_depth },
        }
    }
}

/// Takes ownership of an already boxed provider failure without boxing it again.
impl From<Box<dyn Error + Send + Sync>> for EmbeddedError {
    fn from(source: Box<dyn Error + Send + Sync>) -> Self {
        Self {
            cause: ErrorCause::Provider(source),
        }
    }
}

impl fmt::Display for EmbeddedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            ErrorCause::DepthLimit { max_depth } => {
                write!(
                    formatter,
                    "embedded formatting exceeded the depth limit ({max_depth})"
                )
            }
            ErrorCause::Provider(source) => {
                write!(formatter, "embedded formatter failed: {source}")
            }
        }
    }
}

impl Error for EmbeddedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.cause {
            ErrorCause::DepthLimit { .. } => None,
            ErrorCause::Provider(source) => Some(source.as_ref()),
        }
    }
}
