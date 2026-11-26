async function f() {
  (await // @ts-expect-error
  source).value;
}
