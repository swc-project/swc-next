import type { Doc, Plugin } from "prettier";

interface EmbeddedBlockBase {
  /** Start of the block in UTF-16 code units. */
  start: number;
  /** Exclusive end of the block in UTF-16 code units. */
  end: number;
  language: string;
  body: string;
}

interface Fence extends EmbeddedBlockBase {
  meta?: string;
  sourceInfo?: string;
}

interface FrontMatter extends EmbeddedBlockBase {
  opening: string;
  closing: string;
}

type EmbeddedBlock = Fence | FrontMatter;

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
