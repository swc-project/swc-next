export interface MarkdownOptions {
  proseWrap?: "preserve" | "always" | "never";
  singleQuote?: boolean;
  lineWidth?: number;
  indentStyle?: "space" | "tab";
  indentWidth?: number;
  lineEnding?: "lf" | "crlf" | "cr";
  /** Include labeled embedded blocks for a host to format. Defaults to true. */
  includeEmbeds?: boolean;
}

export interface Fence {
  /** Start of the fence in UTF-16 code units. */
  start: number;
  /** Exclusive end of the fence in UTF-16 code units. */
  end: number;
  language: string;
  meta?: string;
  /** Spelling to print when the decoded info would reparse with different fields. */
  sourceInfo?: string;
  body: string;
}

export interface FrontMatter {
  start: number;
  end: number;
  language: string;
  /** Complete document between delimiter lines, with LF line endings. */
  body: string;
  opening: string;
  closing: string;
}

export type EmbeddedBlock = Fence | FrontMatter;

/** The complete SWCD buffer and its decoded Markdown records in label order. */
export declare function formatDocSync(
  source: string,
  options?: MarkdownOptions,
): { doc: Uint8Array; embeds: EmbeddedBlock[]; readonly fences: Fence[] };
