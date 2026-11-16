function f() {
  throw /* @ts-ignore */ (
    /** @type {number} */ (first)
  )();
}
