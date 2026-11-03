/** Internal, version-matched SWCFMT wire API. Prefer the Prettier plugin. */
export declare function formatDocBufferSync(
  source: string,
  parser: number,
  options: Uint8Array,
): Uint8Array;

/** Layout options for a version-matched SWCD document. */
export interface PrintDocOptions {
  printWidth?: number;
  tabWidth?: number;
  useTabs?: boolean;
  endOfLine?: "lf" | "crlf" | "cr";
}

/** Print an SWCD document with the Rust formatter. */
export declare function printDocBufferSync(
  buffer: Uint8Array,
  options?: PrintDocOptions,
): string;

export interface PrintDocResult {
  formatted: string;
  cursorNodeStart?: number;
  cursorNodeText?: string;
}

/** Prints an SWCD document and reports its emitted cursor range in UTF-16. */
export declare function printDocBufferWithCursorSync(
  buffer: Uint8Array,
  options?: PrintDocOptions,
): PrintDocResult;
