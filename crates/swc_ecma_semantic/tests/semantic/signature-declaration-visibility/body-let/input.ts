const value = "outer";
function f(x: typeof value) {
  let value = 123;
}
