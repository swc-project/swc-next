import { transformSync as __transformSync } from "./binding.js";
import { encodeGeneratorConfig } from "@swc-next/encoder/options";
import { createTransformSync } from "@swc-next/decoder/transform";

export const transformSync = createTransformSync(__transformSync, encodeGeneratorConfig);
