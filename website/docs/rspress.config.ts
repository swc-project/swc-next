import { defineConfig } from "@rspress/core";
import { pluginOpenGraph } from "rsbuild-plugin-open-graph";
import { loadConformanceReport } from "./conformance-report";
import { sidebar } from "./navigation";

export default defineConfig({
  root: "docs",
  builderConfig: {
    plugins: [
      pluginOpenGraph({
        siteName: "SWC Next",
        type: "website",
        url: "https://next.swc.rs/",
        image: "https://next.swc.rs/og.png",
        description: "A fast, extensible JavaScript and TypeScript toolchain, written in Rust.",
        twitter: {
          site: "@swc_rs",
          card: "summary_large_image",
          image: "https://next.swc.rs/og.png",
        },
      }),
    ],
    source: {
      define: {
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
