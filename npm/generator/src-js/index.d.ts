import type { AstNode } from "@swc-next/ast";
import type { Program } from "@swc-next/types";
export type * from "@swc-next/types";

import type { GeneratorConfig, GenerateOutput } from "./binding.js";
export type { GeneratorConfig, GenerateOutput } from "./binding.js";

export type GenerateApiOptions = GeneratorConfig;

export declare function generateSync(
  program: AstNode<Program>,
  options?: GenerateApiOptions | undefined | null,
): GenerateOutput;

export declare function generateBuffersSync(buffer: Uint8Array, extension?: Uint8Array | null, options?: GenerateApiOptions | null): GenerateOutput;
