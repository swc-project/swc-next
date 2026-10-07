/** Source metadata, reusable by the encoder. */
export interface AstMeta {
  readonly lang: "js" | "jsx" | "ts" | "tsx" | "dts";
  readonly sourceType: "module" | "script" | "commonjs" | "unambiguous";
}

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
  readonly meta: AstMeta;
  readonly program: any;
  readonly comments: Comment[];
  readonly diagnostics: Diagnostic[];
}

/** Decode a buffer produced by an SWC Next parser crate into ESTree. */
export declare function decode(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;

/** Decode a JavaScript buffer produced by an SWC Next parser crate. */
export declare function decodeJs(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;

/** Decode a TypeScript buffer produced by an SWC Next parser crate. */
export declare function decodeTs(buffer: Uint8Array, source: string, sourceBuffer: Uint8Array): DecodeResult;
