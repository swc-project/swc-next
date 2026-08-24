import {
  parseExpressionFragmentSync as __parseExpressionFragmentSync,
  parseStatementFragmentSync as __parseStatementFragmentSync,
  parseSync as __parseSync,
} from "./binding.js";
import { decode } from "./decode.js";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, options);
  return decode(buffer, source);
}

export function parseExpressionFragmentSync(source, fragmentContext, options) {
  const buffer = __parseExpressionFragmentSync(source, fragmentContext, options);
  return decode(buffer, source, "expression");
}

export function parseStatementFragmentSync(source, fragmentContext, options) {
  const buffer = __parseStatementFragmentSync(source, fragmentContext, options);
  return decode(buffer, source, "statement");
}
