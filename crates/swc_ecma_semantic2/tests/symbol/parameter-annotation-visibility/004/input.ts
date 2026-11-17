export {};
const value = "outer";
function f(x: typeof value = "outer") {
  var value = 1;
}
