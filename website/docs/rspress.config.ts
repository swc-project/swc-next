import { defineConfig } from "@rspress/core";
import { loadBenchmarkReport } from "./benchmark-report";
import { loadConformanceReport } from "./conformance-report";
import { sidebar } from "./navigation";

export default defineConfig({
  root: "docs",
  builderConfig: {
    source: {
      define: {
        BENCHMARK_REPORT: JSON.stringify(loadBenchmarkReport()),
        CONFORMANCE_REPORT: JSON.stringify(loadConformanceReport()),
      },
    },
  },
  title: "SWC Next",
  description: "A fast, extensible JavaScript and TypeScript toolchain, written in Rust.",
  icon: "/favicon.ico",
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
