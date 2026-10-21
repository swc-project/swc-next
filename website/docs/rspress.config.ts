import { defineConfig } from "@rspress/core";
import { sidebar } from "./navigation";

export default defineConfig({
  root: "docs",
  title: "SWC Next",
  description: "A fast, extensible JavaScript and TypeScript toolchain, written in Rust.",
  icon: "/favicon.svg",
  lang: "en",
  route: { cleanUrls: true },
  llms: true,
  themeConfig: {
    darkMode: "dark",
    nav: [
      { text: "Guide", link: "/", activeMatch: "^/$|^/guide/" },
      {
        text: "API",
        items: [
          { text: "Overview", link: "/api/" },
          { text: "Component APIs", link: "/api/components/" },
          { text: "JavaScript", link: "/api/javascript/core" },
          { text: "Rust", link: "/api/rust" },
          { text: "AST interop", link: "/api/interop" },
        ],
      },
      { text: "Benchmarks", link: "/benchmarks/" },
      {
        text: "Resources",
        items: [
          { text: "Architecture", link: "/resources/architecture/flat-ast" },
          { text: "Testing & conformance", link: "/resources/testing" },
          { text: "Roadmap", link: "/resources/roadmap" },
          { text: "Releases", link: "/resources/releases" },
          { text: "Contributing", link: "/resources/contributing" },
          { text: "FAQ", link: "/resources/faq" },
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
