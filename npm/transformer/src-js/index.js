import { transformSync as __transformSync } from "./binding.js";
import { createTransformSync } from "@swc-next/decoder/transform";

export const transformSync = createTransformSync(__transformSync);
