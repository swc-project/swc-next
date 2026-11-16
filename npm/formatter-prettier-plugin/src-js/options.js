import { options as javascriptOptions } from "prettier/plugins/estree";

// Prettier supplies validated, normalized options to the plugin.
export const options = javascriptOptions;

export function toFormatterConfig(options) {
  if (
    options.experimentalTernaries ||
    options.experimentalOperatorPosition === "start" ||
    options.objectWrap === "collapse"
  ) {
    throw new Error("SWCFMT does not yet support the requested layout option");
  }
  return {
    lineWidth: options.printWidth,
    indentStyle: options.useTabs ? "tab" : "space",
    indentWidth: options.tabWidth,
    lineEnding: options.endOfLine,
    semicolons: options.semi ? "always" : "as-needed",
    quoteStyle: options.singleQuote ? "single" : "double",
    jsxQuoteStyle: options.jsxSingleQuote ? "single" : "double",
    quoteProperties: options.quoteProps,
    trailingCommas: options.trailingComma,
    arrowParentheses: options.arrowParens === "always" ? "always" : "as-needed",
    bracketSpacing: options.bracketSpacing,
    bracketSameLine: options.bracketSameLine || options.jsxBracketSameLine,
    singleAttributePerLine: options.singleAttributePerLine,
  };
}
