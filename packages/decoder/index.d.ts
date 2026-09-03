export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface DecodeResult {
  readonly program: any;
  readonly comments: Comment[];
}

/** Decode a buffer produced by an SWC Next parser crate into ESTree. */
export declare function decode(buffer: Uint8Array, source: string): DecodeResult;
