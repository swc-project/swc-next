import { usePage } from "@rspress/core/runtime";
import { Layout as OriginalLayout, Link, Sidebar as OriginalSidebar } from "@rspress/core/theme-original";
import { ArrowUpRight, BookOpen, Code2 } from "lucide-react";
import "./index.css";

const sectionNames: Record<string, string> = {
  guide: "Guide",
  api: "API reference",
  architecture: "Architecture",
  resources: "Resources",
};

export function Sidebar() {
  return <div className="sidebar-navigation"><OriginalSidebar /></div>;
}

export function Layout() {
  const { page } = usePage();
  const section = page.routePath.split("/").filter(Boolean)[0] ?? "guide";

  return (
    <OriginalLayout
      navTitle={
        <Link className="site-brand" href="/" aria-label="SWC Next documentation home">
          <img src="/images/swc.png" alt="" width="72" height="26" />
        </Link>
      }
      beforeDocContent={
        <div className="doc-eyebrow">
          <BookOpen size={13} />
          <span>{sectionNames[section] ?? "Guide"}</span>
          <span className="doc-eyebrow__separator">/</span>
          <span>{page.title}</span>
        </div>
      }
      afterSidebar={
        <div className="sidebar-footer">
          <a href="https://play.next.swc.rs" target="_blank" rel="noreferrer">
            <ArrowUpRight size={15} /> Open Playground
          </a>
          <a href="https://github.com/swc-project/swc-next" target="_blank" rel="noreferrer">
            <Code2 size={15} /> GitHub repository
          </a>
        </div>
      }
    />
  );
}

export * from "@rspress/core/theme-original";
