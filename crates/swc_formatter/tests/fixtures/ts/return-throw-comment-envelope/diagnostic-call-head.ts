function f() {
  return ((/* @ts-expect-error */
    missing)());
}
