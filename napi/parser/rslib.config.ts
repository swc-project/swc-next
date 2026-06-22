import { defineConfig } from "@rslib/core";

export default defineConfig({
  lib: [
    {
      source: {
        entry: { index: "src-js/index.ts" },
      },
      dts: true,
    },
  ],
  output: {
    cleanDistPath: true,
    copy: {
      patterns: [
        {
          context: "src-js",
          from: "*.node",
        },
      ],
    },
  },
});
