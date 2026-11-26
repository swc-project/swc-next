import type { EmbeddedBlock } from "@swc-next/formatter-markdown";
import type { Doc, Plugin } from "prettier";

interface Root {
  type: "SWCFMT_MARKDOWN";
  start: number;
  end: number;
  doc: Doc;
  embeds: EmbeddedBlock[];
}

type Node = Root | EmbeddedBlock;

export declare const languages: Plugin<Node>["languages"];
export declare const parsers: Plugin<Node>["parsers"];
export declare const printers: Plugin<Node>["printers"];
