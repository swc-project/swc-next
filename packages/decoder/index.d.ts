export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface DiagnosticLabel {
  message: string;
  /** UTF-16 offset in the source. */
  start: number;
  /** UTF-16 offset in the source. */
  end: number;
}

export interface Diagnostic extends DiagnosticLabel {
  severity: "error" | "warning" | "hint" | "info";
  help?: string;
  labels: DiagnosticLabel[];
}

export interface DecodeResult {
  readonly program: any;
  readonly comments: Comment[];
  readonly diagnostics: Diagnostic[];
}

/** Decode a buffer produced by an SWC Next parser crate into ESTree. */
export declare function decode(buffer: Uint8Array, source: string): DecodeResult;

/** Decode a JavaScript buffer produced by an SWC Next parser crate. */
export declare function decode_js(buffer: Uint8Array, source: string): DecodeResult;

/** Decode a TypeScript buffer produced by an SWC Next parser crate. */
export declare function decode_ts(buffer: Uint8Array, source: string): DecodeResult;
