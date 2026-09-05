#!/usr/bin/env node

import { fileURLToPath } from "node:url";

import { prepareBrowserPackage } from "../../../scripts/browser-package.mjs";

prepareBrowserPackage(
  fileURLToPath(new URL("../../../release-dir/core/wasm32-unknown-unknown/", import.meta.url)),
);
