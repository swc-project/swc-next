import { Program } from "./node";
import { measure } from "./generated/measure";
import { Header } from "./protocol";

export class Measurement {
  private textEncoder = new TextEncoder();
  private header: Header = {
    nodeCount: 0,
    extraCount: 0,
    stringPoolSize: 0,
  };

  constructor(private ast: Program) {}

  measure(): Header {
    measure(this, this.ast);
    return this.header;
  }

  node() {
    this.header.nodeCount += 1;
  }

  nodes(nodes: readonly unknown[]) {
    this.header.extraCount += nodes.length;

    for (const node of nodes) {
      if (node != null) {
        measure(this, node);
      }
    }
  }

  string(value: string) {
    this.header.stringPoolSize += this.textEncoder.encode(value).length;
  }

  optString(value: string | null | undefined) {
    if (value != null) {
      this.string(value);
    }
  }
}
