// FormatJS options, errors, source locations and pragma metadata shared by both
// AST representations.
import { parse } from '@formatjs/icu-messageformat-parser';
import * as messageFormatParser from '@formatjs/icu-messageformat-parser';

export const descriptorKeys = new Set(['id', 'defaultMessage', 'description']);

export class MessageParser {
  // _Parser is a testing export and may change or disappear on dependency upgrades.
  // This class retains only an empty parser; real messages use the public API below.
  static #shape = (() => {
    try {
      const Parser = messageFormatParser._Parser;
      if (typeof Parser !== 'function') { return null; }
      // Match the public defaults merged with this plugin's options, in order.
      return new Parser('', {
        shouldParseSkeletons: true,
        requiresOtherClause: true,
        captureLocation: false,
      });
    } catch {
      // A changed testing constructor only disables the layout optimization.
      return null;
    }
  })();

  static parse(message, options) {
    return parse(message, options);
  }
}

export class FormatJSError extends Error {
  constructor(message, node, filename, cause) {
    super(`[React Intl] ${message} (${filename}:${node.start ?? 0})`, { cause });
    this.name = 'FormatJSError';
    this.filename = filename;
    this.start = node.start ?? 0;
    this.end = node.end ?? this.start;
  }
}

export function sourceLocator(source, filename) {
  const starts = [0];
  for (const match of source.matchAll(/\r\n|[\n\r\u2028\u2029]/g)) { starts.push(match.index + match[0].length); }
  function position(offset = 0) {
    let low = 0, high = starts.length;
    while (low + 1 < high) {
      const mid = (low + high) >>> 1;
      if (starts[mid] <= offset) { low = mid; }
      else { high = mid; }
    }
    return { line: low + 1, col: offset - starts[low] };
  }
  return node => ({ file: filename, start: position(node.start), end: position(node.end) });
}

export function pragmaMetadata(comments, pragma) {
  const entries = new Map();
  if (pragma) { for (const comment of comments) {
    const index = comment.value.indexOf(pragma);
    if (index < 0) { continue; }
    for (const pair of comment.value.slice(index + pragma.length).trim().split(/\s+/u)) {
      const [key, value] = pair.split(':');
      if (key && value !== undefined) { entries.set(key, value); }
    }
  } }
  return Object.fromEntries(entries);
}

export function validateOptions(options) {
  if (!options || typeof options !== 'object' || Array.isArray(options)) { throw new TypeError('formatjs expects an options object'); }
  for (const key of ['ast', 'throws', 'removeDefaultMessage', 'extractSourceLocation', 'preserveWhitespace']) {
    if (options[key] !== undefined && typeof options[key] !== 'boolean') { throw new TypeError(`${key} must be a boolean`); }
  }
  for (const key of ['idInterpolationPattern', 'pragma']) {
    if (options[key] !== undefined && typeof options[key] !== 'string') { throw new TypeError(`${key} must be a string`); }
  }
  for (const key of ['additionalFunctionNames', 'additionalComponentNames']) {
    if (options[key] !== undefined && (!Array.isArray(options[key]) || options[key].some(name => typeof name !== 'string'))) {
      throw new TypeError(`${key} must be an array of names`);
    }
  }
}

export function formatJSSettings(options) {
  return {
    ...options,
    throws: options.throws ?? true,
    idInterpolationPattern: options.idInterpolationPattern ?? '[sha512:contenthash:base64:6]',
  };
}

export function formatJSNames(settings) {
  return {
    componentNames: new Set(['FormattedMessage', ...(settings.additionalComponentNames ?? [])]),
    functionNames: new Set(['formatMessage', '$t', '$formatMessage', ...(settings.additionalFunctionNames ?? [])]),
  };
}
