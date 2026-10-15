import {
  Braces,
  ChevronDown,
  Code2,
  RotateCcw,
  Settings2,
  WandSparkles,
} from "lucide-react";
import type { ReactNode } from "react";
import { useAtom, useSetAtom } from "jotai";
import { optionsAtom, resetOptionsAtom } from "../state";
import type { Options } from "../types";

function Toggle({
  label,
  checked,
  onChange,
  disabled,
  hint,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
  hint?: string;
}) {
  return (
    <label
      className={`flex min-h-10 items-center justify-between gap-2 text-sm/[21px] max-[720px]:gap-3 ${disabled ? "cursor-default text-subtle" : "cursor-pointer"}`}
      title={hint}
    >
      <span>{label}</span>
      <input
        className="relative m-0 h-4.5 w-7.5 shrink-0 cursor-inherit appearance-none rounded-full border border-border bg-hover transition-colors before:absolute before:top-0.5 before:left-0.5 before:size-3 before:rounded-full before:bg-muted before:transition-transform before:content-[''] checked:border-ring checked:bg-ring checked:before:translate-x-3 checked:before:bg-white disabled:opacity-30 motion-reduce:transition-none motion-reduce:before:transition-none"
        type="checkbox"
        role="switch"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        disabled={disabled}
      />
    </label>
  );
}

function Group({
  title,
  icon,
  children,
}: {
  title: string;
  icon: ReactNode;
  children: ReactNode;
}) {
  return (
    <details
      className="group/options px-5 open:pb-5 max-[720px]:open:pb-3"
      open
    >
      <summary className="flex h-11 items-center gap-2 text-sm/[21px] font-semibold text-heading [&>svg:first-child]:text-muted">
        {icon}
        <span>{title}</span>
        <ChevronDown
          size={13}
          className="ml-auto text-subtle -rotate-90 transition-transform group-open/options:rotate-0 motion-reduce:transition-none"
        />
      </summary>
      <div className="flex flex-col gap-1 max-[720px]:grid max-[720px]:grid-cols-2 max-[720px]:gap-x-5">
        {children}
      </div>
    </details>
  );
}

export default function OptionsPanel() {
  const [options, onChange] = useAtom(optionsAtom);
  const onReset = useSetAtom(resetOptionsAtom);
  const parser = (patch: Partial<Options["parser"]>) =>
    onChange({ ...options, parser: { ...options.parser, ...patch } });
  const codegen = (patch: Partial<Options["codegen"]>) =>
    onChange({ ...options, codegen: { ...options.codegen, ...patch } });
  const typescript = ["ts", "tsx", "dts"].includes(options.parser.lang);
  return (
    <aside
      className="flex min-h-0 flex-col border-r border-border bg-sidebar max-[720px]:border-r-0 max-[720px]:border-b"
      aria-label="Compiler configuration"
    >
      <div className="flex h-13 min-h-13 items-center justify-between gap-2.5 border-b border-border px-5 text-sm/[21px] font-medium text-foreground max-[1000px]:px-4">
        <span className="flex min-w-0 items-center gap-2 whitespace-nowrap [&>svg:first-child]:text-muted">
          <Settings2 size={14} />
          Configuration
        </span>
        <button
          className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
          title="Reset options"
          aria-label="Reset options"
          onClick={onReset}
        >
          <RotateCcw size={13} />
        </button>
      </div>
      <div className="flex-1 overflow-auto pt-3 pb-7 max-[720px]:overflow-visible max-[720px]:py-2">
        <Group title="Parser" icon={<Braces size={14} />}>
          <label className="mt-1 mb-3 flex flex-col gap-2 text-sm/[21px] text-muted">
            Language
            <select
              className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
              aria-label="Language"
              value={options.parser.lang}
              onChange={(event) =>
                parser({
                  lang: event.target.value as Options["parser"]["lang"],
                })
              }
            >
              <option value="js">JavaScript</option>
              <option value="jsx">JavaScript + JSX</option>
              <option value="ts">TypeScript</option>
              <option value="tsx">TypeScript + JSX</option>
              <option value="dts">TypeScript declarations</option>
            </select>
          </label>
          <label className="mt-1 mb-3 flex flex-col gap-2 text-sm/[21px] text-muted">
            Source type
            <select
              className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
              aria-label="Source type"
              value={options.parser.sourceType}
              onChange={(event) =>
                parser({
                  sourceType: event.target
                    .value as Options["parser"]["sourceType"],
                })
              }
            >
              <option value="module">Module</option>
              <option value="script">Script</option>
              <option value="commonjs">CommonJS</option>
              <option value="unambiguous">Unambiguous</option>
            </select>
          </label>
          <Toggle
            label="Preserve parentheses"
            checked={options.parser.preserveParens}
            onChange={(preserveParens) => parser({ preserveParens })}
          />
          <Toggle
            label="Semantic errors"
            checked={options.parser.semanticErrors}
            onChange={(semanticErrors) => parser({ semanticErrors })}
            hint="Check scope-dependent early errors. Required checks also run before transforms."
          />
        </Group>
        <Group title="Transform" icon={<WandSparkles size={14} />}>
          <Toggle
            label="TypeScript"
            checked={options.transform.ts}
            disabled={!typescript}
            onChange={(ts) =>
              onChange({ ...options, transform: { ...options.transform, ts } })
            }
            hint="Lower TypeScript syntax to JavaScript; JSX is preserved."
          />
          <details className="group/target mt-3 max-[720px]:mt-0" open>
            <summary className="flex h-8.5 items-center gap-1.5 text-sm/[21px] text-muted">
              <ChevronDown
                size={12}
                className="-rotate-90 group-open/target:rotate-0"
              />
              <span>ES2022</span>
              <span className="ml-auto text-xs text-subtle">1</span>
            </summary>
            <div className="mt-0.75 ml-1.25 border-l border-border pl-3.5 max-[720px]:pl-3 max-[720px]:[&>label]:items-start">
              <Toggle
                label="Class static block"
                checked={options.transform.es2022.classStaticBlock}
                onChange={(classStaticBlock) =>
                  onChange({
                    ...options,
                    transform: {
                      ...options.transform,
                      es2022: { classStaticBlock },
                    },
                  })
                }
              />
            </div>
          </details>
        </Group>
        <Group title="Codegen" icon={<Code2 size={14} />}>
          <Toggle
            label="Minify"
            checked={options.codegen.minify}
            onChange={(minify) => codegen({ minify })}
            hint="Print compact code without extra whitespace."
          />
          <Toggle
            label="ASCII only"
            checked={options.codegen.asciiOnly}
            onChange={(asciiOnly) => codegen({ asciiOnly })}
          />
          <Toggle
            label="Inline script"
            checked={options.codegen.inlineScript}
            onChange={(inlineScript) => codegen({ inlineScript })}
            hint="Escape closing script tags for HTML embedding."
          />
        </Group>
      </div>
      <div className="flex h-10 min-h-10 items-center gap-1.75 border-t border-border px-5 text-xs text-muted max-[720px]:hidden">
        <span className="size-1.25 shrink-0 rounded-full bg-success" />
        <span>Rust + WebAssembly</span>
        <span className="ml-auto text-[10px] tracking-[0.7px] text-subtle">
          LOCAL
        </span>
      </div>
    </aside>
  );
}
