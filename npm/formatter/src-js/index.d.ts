/** Layout options for a version-matched SWCD document. */
export interface PrintDocOptions {
  /** @default 80 */
  lineWidth?: number;
  /** @default 2 */
  indentWidth?: number;
  /** @default "space" */
  indentStyle?: "tab" | "space";
  /** @default "lf" */
  lineEnding?: "lf" | "crlf" | "cr";
}

/** Print an SWCD document with the Rust formatter. */
export declare function printDocBufferSync(
  buffer: Uint8Array,
  options?: PrintDocOptions | null,
): string;

export interface PrintDocResult {
  formatted: string;
  cursorNodeStart?: number;
  cursorNodeText?: string;
}

/** Prints an SWCD document and reports its emitted cursor range in UTF-16. */
export declare function printDocBufferWithCursorSync(
  buffer: Uint8Array,
  options?: PrintDocOptions | null,
): PrintDocResult;
