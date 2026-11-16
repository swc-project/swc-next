/** Internal, version-matched SWC formatter transport. */
export declare function encodePrintConfig(options?: object | null): Uint8Array;
export declare function encodeFormatterConfig(
  options?: object | null,
): Uint8Array;
export declare function decodeFormatDocOutput(
  buffer: Uint8Array,
  source: string,
):
  | {
      buffer: Uint8Array;
      error?: never;
    }
  | {
      buffer?: never;
      error: { message: string; start: number; end: number };
    };
export declare function decodePrintDocOutput(buffer: Uint8Array): {
  formatted: string;
  cursorNodeStart?: number;
  cursorNodeText?: string;
};
