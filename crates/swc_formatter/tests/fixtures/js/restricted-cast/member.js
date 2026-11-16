function f() {
  return /* @ts-ignore */ (
    /** @type {number} */ (first)
  ).b;
}
