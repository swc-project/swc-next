import { defineConfig } from "@rspress/core";
import { loadConformanceReport } from "./conformance-report";
import { sidebar } from "./navigation";

export default defineConfig({
  root: "docs",
  builderConfig: {
    source: {
      define: {
        CONFORMANCE_REPORT: JSON.stringify(loadConformanceReport()),
      },
    },
  },
  title: "SWC Next",
  description: "A fast, extensible JavaScript and TypeScript toolchain, written in Rust.",
  icon: "/favicon.ico",
  head: [
    ["meta", { property: "og:image", content: "https://next.swc.rs/og.png" }],
    ["meta", { property: "og:image:width", content: "1200" }],
    ["meta", { property: "og:image:height", content: "630" }],
    ["meta", { property: "og:image:type", content: "image/png" }],
    ["meta", { property: "og:image:alt", content: "SWC Next — JavaScript and TypeScript toolchain" }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:image", content: "https://next.swc.rs/og.png" }],
    ["meta", { name: "twitter:image:alt", content: "SWC Next — JavaScript and TypeScript toolchain" }],
  ],
  lang: "en",
  route: { cleanUrls: true },
  llms: true,
  themeConfig: {
    darkMode: "auto",
    nav: [
      { text: "Guide", link: "/", activeMatch: "^/$|^/guide/" },
      { text: "API", link: "/api/" },
      { text: "Architecture", link: "/architecture/flat-ast", activeMatch: "^/architecture/" },
      {
        text: "Resources",
        items: [
          { text: "Team", link: "/resources/team" },
          { text: "Sponsors", link: "/resources/sponsors" },
          { text: "Customers", link: "/resources/customers" },
          { text: "Roadmap", link: "https://github.com/swc-project/swc-next/issues/181" },
          { text: "Releases", link: "https://github.com/swc-project/swc-next/releases" },
        ],
      },
    ],
    sidebar,
    socialLinks: [
      { icon: "github", mode: "link", content: "https://github.com/swc-project/swc-next" },
    ],
    editLink: {
      docRepoBaseUrl: "https://github.com/swc-project/swc-next/tree/main/website/docs/docs",
    },
    lastUpdated: false,
    llmsUI: { placement: "outline", viewOptions: ["markdownLink"], injectLlmsHint: false },
  },
});
