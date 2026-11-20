function f(
  arg = () => {
    const value = 1;
    return undefined as unknown as typeof value;
  },
) {
  function value() {}
}
