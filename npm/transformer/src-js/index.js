import { transformSync as __transformSync } from "./binding.js";
import { createTransformSync } from "@swc-next/transformer-bridge";

export const transformSync = createTransformSync(__transformSync);
