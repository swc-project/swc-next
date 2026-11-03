function f() {
  const fn = function inner() { return inner; };
  const C = class Inner { method() { return Inner; } };
  return [fn, C];
}
