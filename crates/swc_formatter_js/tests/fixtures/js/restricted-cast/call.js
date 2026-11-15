function f() {
  return /* @ts-ignore */ (
    /** @type {Function} */ (first)
  )();
}
