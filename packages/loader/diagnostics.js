export class SwcNextLoaderError extends Error {
  constructor(message, diagnostics = []) {
    super(message);
    this.name = "SwcNextLoaderError";
    this.diagnostics = diagnostics;
  }
}

function formatDiagnostic(diagnostic, source, filename) {
  let prefix = filename ? `${filename}: ` : "";
  if (source.length > 0 && (diagnostic.start > 0 || diagnostic.end > 0)) {
    const before = source.slice(0, Math.min(diagnostic.start, source.length));
    const lines = before.split(/\r\n?|\n/);
    const loc = `${lines.length}:${lines.at(-1).length + 1}`;
    prefix = filename ? `${filename}:${loc}: ` : `${loc}: `;
  }
  return diagnostic.help
    ? `${prefix}${diagnostic.message}\nhelp: ${diagnostic.help}`
    : `${prefix}${diagnostic.message}`;
}

export function reportDiagnostics(context, diagnostics, source, filename) {
  if (diagnostics.length === 0) return null;

  const errors = [];
  for (const diagnostic of diagnostics) {
    if (diagnostic.severity === "error") {
      errors.push(diagnostic);
    } else if (diagnostic.severity === "warning") {
      context.emitWarning?.(
        new SwcNextLoaderError(
          formatDiagnostic(diagnostic, source, filename),
          [diagnostic],
        ),
      );
    }
  }

  if (errors.length === 0) return null;
  const message = errors
    .map((diagnostic) => formatDiagnostic(diagnostic, source, filename))
    .join("\n");
  return new SwcNextLoaderError(message, errors);
}
