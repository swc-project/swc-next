function f(source, target) {
  const { key, alias = 1, ...rest } = source;
  ({ target } = source);
  return { key, alias, rest, target };
}
