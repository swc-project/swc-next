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
import type {
  AssumptionsOptions,
  DecoratorVersionOption,
  JsxRuntimeOption,
  ModuleOption,
  Options,
  TargetOption,
} from "../types";

const ASSUMPTION_ITEMS: ReadonlyArray<{
  key: keyof AssumptionsOptions;
  label: string;
}> = [
  { key: "constantSuper", label: "Constant super" },
  { key: "ignoreFunctionLength", label: "Ignore function length" },
  { key: "noDocumentAll", label: "No document.all" },
  { key: "objectRestNoSymbols", label: "Object rest no symbols" },
  { key: "privateFieldsAsProperties", label: "Private fields as properties" },
  { key: "pureGetters", label: "Pure getters" },
  { key: "setPublicClassFields", label: "Set public class fields" },
  { key: "setSpreadProperties", label: "Set spread properties" },
];

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
  const transformer = (patch: Partial<Options["transformer"]>) =>
    onChange({ ...options, transformer: { ...options.transformer, ...patch } });
  const generator = (patch: Partial<Options["generator"]>) =>
    onChange({ ...options, generator: { ...options.generator, ...patch } });
  const isTs = ["ts", "tsx", "dts"].includes(options.parser.lang);
  const isJsx = ["jsx", "tsx"].includes(options.parser.lang);
  const { typescript, jsx, decorators, assumptions } = options.transformer;
  const activeAssumptions = Object.values(assumptions).filter(Boolean).length;
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
            hint="Check scope-dependent early errors."
          />
        </Group>
        <Group title="Transformer" icon={<WandSparkles size={14} />}>
          <label className="mt-1 mb-3 flex flex-col gap-2 text-sm/[21px] text-muted">
            Target
            <select
              className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
              aria-label="Target"
              value={options.transformer.target}
              onChange={(event) =>
                transformer({ target: event.target.value as TargetOption })
              }
            >
              <option value="esnext">ESNext</option>
              <option value="es2022">ES2022</option>
              <option value="es2021">ES2021</option>
              <option value="es2020">ES2020</option>
              <option value="es2019">ES2019</option>
              <option value="es2018">ES2018</option>
              <option value="es2017">ES2017</option>
              <option value="es2016">ES2016</option>
              <option value="es2015">ES2015</option>
            </select>
          </label>
          <label className="mt-1 mb-3 flex flex-col gap-2 text-sm/[21px] text-muted">
            Module
            <select
              className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
              aria-label="Module"
              value={options.transformer.module}
              onChange={(event) =>
                transformer({ module: event.target.value as ModuleOption })
              }
            >
              <option value="preserve">Preserve (ESM)</option>
              <option value="commonjs">CommonJS</option>
              <option value="amd">AMD</option>
              <option value="umd">UMD</option>
              <option value="systemjs">SystemJS</option>
            </select>
          </label>
          <Toggle
            label="TypeScript"
            checked={typescript.enabled}
            disabled={!isTs}
            onChange={(enabled) =>
              transformer({ typescript: { ...typescript, enabled } })
            }
            hint="Lower TypeScript syntax to JavaScript."
          />
          {isTs && typescript.enabled && (
            <div className="mb-1 ml-1.25 border-l border-border pl-3.5 max-[720px]:pl-3">
              <Toggle
                label="Verbatim module syntax"
                checked={typescript.verbatimModuleSyntax}
                onChange={(verbatimModuleSyntax) =>
                  transformer({
                    typescript: { ...typescript, verbatimModuleSyntax },
                  })
                }
                hint="Preserve value imports/exports and keep const enums."
              />
            </div>
          )}
          <Toggle
            label="JSX"
            checked={jsx.enabled}
            disabled={!isJsx}
            onChange={(enabled) => transformer({ jsx: { ...jsx, enabled } })}
            hint="Lower JSX syntax to function calls."
          />
          {isJsx && jsx.enabled && (
            <div className="mb-1 ml-1.25 flex flex-col gap-1 border-l border-border pl-3.5 max-[720px]:pl-3">
              <label className="mt-1 mb-2 flex flex-col gap-2 text-sm/[21px] text-muted">
                JSX runtime
                <select
                  className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
                  aria-label="JSX runtime"
                  value={jsx.runtime}
                  onChange={(event) =>
                    transformer({
                      jsx: {
                        ...jsx,
                        runtime: event.target.value as JsxRuntimeOption,
                      },
                    })
                  }
                >
                  <option value="automatic">Automatic</option>
                  <option value="classic">Classic</option>
                </select>
              </label>
              <Toggle
                label="Development"
                checked={jsx.development}
                onChange={(development) =>
                  transformer({ jsx: { ...jsx, development } })
                }
              />
              <Toggle
                label="Source & self"
                checked={jsx.sourceSelf}
                disabled={!jsx.development}
                onChange={(sourceSelf) =>
                  transformer({ jsx: { ...jsx, sourceSelf } })
                }
                hint="Add __source and __self props in development mode."
              />
            </div>
          )}
          <Toggle
            label="Decorators"
            checked={decorators.enabled}
            onChange={(enabled) =>
              transformer({ decorators: { ...decorators, enabled } })
            }
            hint="Lower decorator syntax."
          />
          {decorators.enabled && (
            <div className="mb-1 ml-1.25 flex flex-col gap-1 border-l border-border pl-3.5 max-[720px]:pl-3">
              <label className="mt-1 mb-2 flex flex-col gap-2 text-sm/[21px] text-muted">
                Decorator version
                <select
                  className="h-9.5 w-full cursor-pointer rounded-md border border-border bg-input px-3 text-sm text-foreground hover:border-subtle"
                  aria-label="Decorator version"
                  value={decorators.version}
                  onChange={(event) =>
                    transformer({
                      decorators: {
                        ...decorators,
                        version: event.target.value as DecoratorVersionOption,
                      },
                    })
                  }
                >
                  <option value="2023-11">2023-11</option>
                  <option value="legacy">Legacy</option>
                </select>
              </label>
              <Toggle
                label="Emit metadata"
                checked={decorators.metadata}
                disabled={decorators.version !== "legacy"}
                onChange={(metadata) =>
                  transformer({ decorators: { ...decorators, metadata } })
                }
                hint="Emit Reflect.metadata design-type metadata (legacy only)."
              />
            </div>
          )}
          <details className="group/assumptions mt-2 max-[720px]:mt-0">
            <summary className="flex h-8.5 items-center gap-1.5 text-sm/[21px] text-muted">
              <ChevronDown
                size={12}
                className="-rotate-90 group-open/assumptions:rotate-0"
              />
              <span>Assumptions</span>
              {activeAssumptions > 0 && (
                <span className="ml-auto text-xs text-subtle">
                  {activeAssumptions}
                </span>
              )}
            </summary>
            <div className="mt-0.75 ml-1.25 border-l border-border pl-3.5 max-[720px]:pl-3 max-[720px]:[&>label]:items-start">
              {ASSUMPTION_ITEMS.map(({ key, label }) => (
                <Toggle
                  key={key}
                  label={label}
                  checked={assumptions[key]}
                  onChange={(value) =>
                    transformer({
                      assumptions: { ...assumptions, [key]: value },
                    })
                  }
                />
              ))}
            </div>
          </details>
        </Group>
        <Group title="Generator" icon={<Code2 size={14} />}>
          <Toggle
            label="Comments"
            checked={options.generator.comments}
            onChange={(comments) => generator({ comments })}
            hint="Preserve comments in generated code."
          />
          <Toggle
            label="Minify"
            checked={options.generator.minify}
            onChange={(minify) => generator({ minify })}
            hint="Print compact code without extra whitespace."
          />
          <Toggle
            label="ASCII only"
            checked={options.generator.asciiOnly}
            onChange={(asciiOnly) => generator({ asciiOnly })}
          />
          <Toggle
            label="Inline script"
            checked={options.generator.inlineScript}
            onChange={(inlineScript) => generator({ inlineScript })}
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
