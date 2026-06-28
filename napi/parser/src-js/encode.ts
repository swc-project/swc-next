import { Program, Span } from "./node";
import { encode } from "./generated/encode";
import { EXTRA_SIZE, HEADER_SIZE, NODE_SIZE, NONE } from "./generated/protocol";
import { Header, headerSize, PackedNode } from "./protocol";

export class Encoder {
  textEncoder = new TextEncoder();
  buf: Buffer;

  private dv: DataView;
  private u8: Uint8Array;
  private header: Header;
  private nodeOffset: number;
  private extraOffset: number;
  private stringPoolOffset: number;
  private nodeIndex = 0;
  private extraIndex = 0;
  private stringPoolByteIndex = 0;
  private scratch: number[] = [];

  constructor(header: Header, buf: Buffer) {
    this.header = header;
    this.buf = buf;
    if (this.buf.byteLength < headerSize(this.header)) {
      throw new RangeError("Encoder buffer is too small");
    }

    this.dv = new DataView(this.buf.buffer);
    this.u8 = new Uint8Array(this.buf);
    this.nodeOffset = HEADER_SIZE;
    this.extraOffset = this.nodeOffset + this.header.nodeCount * NODE_SIZE;
    this.stringPoolOffset =
      this.extraOffset + this.header.extraCount * EXTRA_SIZE;
    this.writeHeader(this.header);
  }

  encode(ast: Program) {
    return encode(this, ast);
  }

  packedNode(node: PackedNode): number {
    const offset = this.nodeOffset + this.nodeIndex * NODE_SIZE;

    this.u8[offset] = node.kind;
    this.u8[offset + 1] = 0;
    this.dv.setUint16(offset + 2, node.flags ?? 0, true);
    this.dv.setUint32(offset + 4, node.span_lo ?? 0, true);
    this.dv.setUint32(offset + 8, node.span_hi ?? 0, true);
    this.dv.setUint32(offset + 12, node.field0 ?? 0, true);
    this.dv.setUint32(offset + 16, node.field1 ?? 0, true);
    this.dv.setUint32(offset + 20, node.field2 ?? 0, true);
    this.dv.setUint32(offset + 24, node.field3 ?? 0, true);
    this.dv.setUint32(offset + 28, node.field4 ?? 0, true);

    this.nodeIndex += 1;
    return this.nodeIndex;
  }

  nodes(nodes: unknown[]): [number, number] {
    const scratchStart = this.scratch.length;

    for (const node of nodes) {
      if (node == null) {
        this.scratch.push(NONE);
        continue;
      }
      this.scratch.push(encode(this, node));
    }

    const start = this.extraIndex;
    const len = nodes.length;

    for (let i = scratchStart; i < this.scratch.length; i++) {
      this.writeExtra(this.scratch[i]);
    }

    this.scratch.length = scratchStart;
    return [start, len];
  }

  span(span: Span): [number, number] {
    return [this.utf16ToUtf8(span.start), this.utf16ToUtf8(span.end)];
  }

  optSpan(span: Span | null | undefined): [number, number] {
    return span == null ? [NONE, 0] : this.span(span);
  }

  utf8(atom: string): [number, number] {
    return this.string(atom);
  }

  optUtf8(atom: string | null | undefined): [number, number] {
    return atom == null ? [NONE, 0] : this.utf8(atom);
  }
  wtf8(atom: string): [number, number] {
    return this.string(atom);
  }

  optWtf8(atom: string | null | undefined): [number, number] {
    return atom == null ? [NONE, 0] : this.wtf8(atom);
  }

  private string(value: string): [number, number] {
    // encodeInto
    const bytes = this.textEncoder.encode(value);
    const start = this.stringPoolByteIndex;
    const offset = this.stringPoolOffset + start;

    this.u8.set(bytes, offset);
    this.stringPoolByteIndex += bytes.length;

    return [start, start + bytes.length];
  }

  private writeExtra(value: number) {
    const offset = this.extraOffset + this.extraIndex * EXTRA_SIZE;
    this.dv.setUint32(offset, value, true);
    this.extraIndex += 1;
  }

  private writeHeader(header: Header) {
    this.dv.setUint32(0, header.nodeCount, true);
    this.dv.setUint32(4, header.extraCount, true);
    this.dv.setUint32(8, header.stringPoolSize, true);
  }

  private utf16ToUtf8(offset: number): number {
    // todo
    return offset;
  }
}
