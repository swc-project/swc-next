export {};
const value = "outer";
function f(x: typeof value) {
  var value = 1;
}
