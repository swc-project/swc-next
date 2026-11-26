// Option validation and pattern compilation shared by both implementations.
// `Array.isArray(properties)` is genuine configuration validation, not an AST
// representation check.
export function reactRemovePropertiesPatterns(options = true) {
  if (options === false) { return null; }
  if (options !== true && (options === null || typeof options !== 'object' || Array.isArray(options))) {
    throw new TypeError('reactRemoveProperties expects a boolean or { properties: string[] }');
  }
  const properties = options === true ? [] : options.properties ?? [];
  if (!Array.isArray(properties) || properties.some(p => typeof p !== 'string')) {
    throw new TypeError('properties must be an array of regular expression strings');
  }
  return (properties.length ? properties : ['^data-test']).map(pattern => new RegExp(pattern, 'u'));
}
