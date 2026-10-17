import type { Doc, Plugin } from "prettier";

interface Root {
  type: "SWCFMT";
  start: number;
  end: number;
  doc: Doc;
}

export declare const languages: Plugin<Root>["languages"];
export declare const parsers: Plugin<Root>["parsers"];
export declare const printers: Plugin<Root>["printers"];
export declare const options: Plugin<Root>["options"];
