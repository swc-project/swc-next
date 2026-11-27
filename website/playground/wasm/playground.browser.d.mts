export default function init(): Promise<void>;

export function parseSync(
  source: Uint8Array,
  sourceUtf16Len: number,
  options: number,
): Uint8Array;

export function transformSync(
  source: Uint8Array,
  sourceUtf16Len: number,
  options: Uint8Array,
): Uint8Array;
