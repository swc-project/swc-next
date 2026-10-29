function* f() {
  yield /* @ts-ignore */ (
    /** @type {number} */ (first)
  ) + 1;
}
