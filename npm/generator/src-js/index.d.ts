import type { AstMeta } from "@swc-next/encoder";
export type { AstMeta } from "@swc-next/encoder";
import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";

import type { GeneratorConfig, GenerateOutput } from "./binding.js";
export type { GeneratorConfig, GenerateOutput } from "./binding.js";

export type GenerateApiOptions = GeneratorConfig;

export declare function generateSync(
  program: Program,
  options?: GenerateApiOptions | undefined | null,
  meta?: Partial<AstMeta>,
): GenerateOutput;
