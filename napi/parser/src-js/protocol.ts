import { HEADER_SIZE, NODE_SIZE, EXTRA_SIZE, NONE } from "./generated/protocol";

export interface Header {
  nodeCount: number;
  extraCount: number;
  stringPoolSize: number;
}

export function headerSize(header: Header): number {
  return (
    HEADER_SIZE +
    header.nodeCount * NODE_SIZE +
    header.extraCount * EXTRA_SIZE +
    header.stringPoolSize
  );
}

export interface PackedNode {
  kind: number;
  flags: number;
  span_lo: number;
  span_hi: number;
  field0: number;
  field1: number;
  field2: number;
  field3: number;
  field4: number;
}
