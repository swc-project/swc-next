const value = "outer";
function f(x: typeof value) {
  const value = 123;
}
