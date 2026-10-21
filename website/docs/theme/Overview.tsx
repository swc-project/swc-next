import { Link } from "@rspress/core/theme-original";
import { ArrowRight, ArrowUpRight, Braces, Code2, GitBranch, Layers, Terminal } from "lucide-react";

const paths = [
  { icon: Terminal, title: "Get started", description: "Your first parse, edit, and print.", href: "/guide/getting-started/node", label: "01" },
  { icon: Braces, title: "Explore the tools", description: "Find the components for your pipeline.", href: "/tools/", label: "02" },
  { icon: Code2, title: "Read the API", description: "JavaScript interfaces and Rust entry points.", href: "/api/", label: "03" },
  { icon: Layers, title: "Under the hood", description: "Discover the flat AST architecture.", href: "/resources/architecture/flat-ast", label: "04" },
];

export function IntroActions() {
  return (
    <div className="intro-actions rp-not-doc">
      <Link className="action-primary" href="/guide/getting-started/node">Get started <ArrowRight size={14} /></Link>
      <a className="action-secondary" href="https://play.next.swc.rs" target="_blank" rel="noreferrer">Try the Playground <ArrowUpRight size={14} /></a>
    </div>
  );
}

export function Pipeline() {
  return (
    <div className="pipeline rp-not-doc" aria-label="Source code to AST to your tool to output">
      <div className="pipeline__top"><span><span className="status-dot" /> THE COMPILER TOOLKIT</span><span>Rust at the core. JavaScript in your hands.</span></div>
      <div className="pipeline__flow">
        <div><Code2 size={21} /><strong>Source</strong><span>JavaScript · TypeScript</span></div>
        <ArrowRight className="pipeline__arrow" size={17} />
        <div><Braces size={21} /><strong>AST</strong><span>Parse & analyze</span></div>
        <ArrowRight className="pipeline__arrow" size={17} />
        <div className="pipeline__highlight"><GitBranch size={21} /><strong>Your tool</strong><span>Inspect & transform</span></div>
        <ArrowRight className="pipeline__arrow" size={17} />
        <div><Terminal size={21} /><strong>Output</strong><span>Print & optimize</span></div>
      </div>
      <div className="pipeline__bottom"><code>parse <span>→</span> analyze <span>→</span> transform <span>→</span> print</code><span>COMPOSABLE BY DESIGN</span></div>
    </div>
  );
}

export function StartCards() {
  return (
    <div className="start-cards rp-not-doc">
      {paths.map(({ icon: Icon, title, description, href, label }) => (
        <Link className="start-card" href={href} key={href}>
          <div className="start-card__top"><Icon size={19} /><span>{label}</span></div>
          <div className="start-card__title">{title}<ArrowUpRight size={15} /></div>
          <p>{description}</p>
        </Link>
      ))}
    </div>
  );
}

export function ProjectNote() {
  return (
    <div className="project-note rp-not-doc">
      <span className="status-dot" />
      <div><strong>Built in the open. Moving forward.</strong><p>SWC Next is an active rewrite of SWC. Check the current capabilities and API boundaries before integrating.</p><Link href="/guide/status">View project status <ArrowRight size={13} /></Link></div>
    </div>
  );
}
