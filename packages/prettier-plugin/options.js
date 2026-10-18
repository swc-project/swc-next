import { options as javascriptOptions } from "prettier/plugins/estree";

// Reuse Prettier's option declarations so defaults and validation stay identical.
export const options = javascriptOptions;

export function encodeOptions(options) {
  const { printWidth, tabWidth } = options;
  if (!Number.isInteger(printWidth) || printWidth < 0 || printWidth > 65535) {
    throw new RangeError("SWCFMT printWidth must be between 0 and 65535");
  }
  if (!Number.isInteger(tabWidth) || tabWidth < 0 || tabWidth > 64) {
    throw new RangeError("SWCFMT tabWidth must be between 0 and 64");
  }
  if (
    options.experimentalTernaries ||
    options.experimentalOperatorPosition === "start" ||
    options.objectWrap === "collapse"
  ) {
    throw new Error("SWCFMT does not yet support the requested layout option");
  }
  return Uint8Array.of(
    0,
    printWidth & 255,
    printWidth >>> 8,
    tabWidth,
    Number(options.useTabs),
    Number(options.semi),
    Number(options.singleQuote),
    Number(options.jsxSingleQuote),
    ["as-needed", "preserve", "consistent"].indexOf(options.quoteProps),
    ["all", "es5", "none"].indexOf(options.trailingComma),
    Number(options.arrowParens === "always"),
    Number(options.bracketSpacing),
    Number(options.bracketSameLine || options.jsxBracketSameLine),
    Number(options.singleAttributePerLine),
    ["lf", "crlf", "cr"].indexOf(options.endOfLine),
  );
}
