import { readDocument } from "@swc-next/protocol/document";

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const proseWrapModes = ["preserve", "always", "never"];
const lineEndings = ["lf", "crlf", "cr"];
const invalidBlock = () => new Error("Invalid Markdown embedded block");

export function formatMarkdownDoc(formatBuffer, source, options = {}) {
  if (typeof source !== "string")
    throw new TypeError("source must be a string");
  const {
    proseWrap = "preserve",
    singleQuote = false,
    lineWidth = 80,
    indentStyle = "space",
    indentWidth = 2,
    lineEnding = "lf",
    includeEmbeds = true,
  } = options;
  const mode = proseWrapModes.indexOf(proseWrap);
  const ending = lineEndings.indexOf(lineEnding);
  if (mode < 0) throw new TypeError("invalid Markdown proseWrap");
  if (ending < 0) throw new TypeError("invalid Markdown lineEnding");
  if (typeof singleQuote !== "boolean") {
    throw new TypeError("singleQuote must be a boolean");
  }
  if (typeof includeEmbeds !== "boolean") {
    throw new TypeError("includeEmbeds must be a boolean");
  }
  if (!Number.isInteger(lineWidth) || lineWidth < 0 || lineWidth > 65535) {
    throw new RangeError("invalid Markdown lineWidth");
  }
  if (!Number.isInteger(indentWidth) || indentWidth < 0 || indentWidth > 64) {
    throw new RangeError("invalid Markdown indentWidth");
  }
  if (indentStyle !== "space" && indentStyle !== "tab") {
    throw new TypeError("invalid Markdown indentStyle");
  }
  const doc = formatBuffer(
    source,
    mode,
    singleQuote,
    lineWidth,
    indentWidth,
    indentStyle === "tab",
    ending,
    includeEmbeds,
  );
  const embeds = includeEmbeds
    ? decodeBlocks(readDocument(doc).labelData, source)
    : [];
  return { doc, embeds };
}

function decodeBlocks(records, source) {
  // Blocks arrive in source order. Convert their UTF-8 ranges in one pass.
  let sourceByte = 0;
  let sourceOffset = 0;
  const sourcePosition = (byte) => {
    while (sourceByte < byte && sourceOffset < source.length) {
      const scalar = source.codePointAt(sourceOffset);
      sourceByte +=
        scalar < 0x80 ? 1 : scalar < 0x800 ? 2 : scalar < 0x10000 ? 3 : 4;
      sourceOffset += scalar > 0xffff ? 2 : 1;
    }
    if (sourceByte !== byte) throw invalidBlock();
    return sourceOffset;
  };

  return records.map((bytes) => {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let offset = 0;
    const take = (length) => {
      const start = offset;
      offset += length;
      if (offset > bytes.length) throw invalidBlock();
      return start;
    };
    const word = () => view.getUint32(take(4), true);
    const text = () => {
      const start = take(word());
      return utf8.decode(bytes.subarray(start, offset));
    };

    const kind = word();
    const block = {
      start: sourcePosition(word()),
      end: sourcePosition(word()),
      language: text(),
      body: text(),
    };
    if (kind === 0) {
      block.meta = text() || undefined;
      if (offset < bytes.length) block.sourceInfo = text();
    } else if (kind === 1) {
      block.opening = text();
      block.closing = text();
    } else throw invalidBlock();
    if (offset !== bytes.length) throw invalidBlock();
    return block;
  });
}
