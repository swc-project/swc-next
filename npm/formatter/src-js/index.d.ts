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
