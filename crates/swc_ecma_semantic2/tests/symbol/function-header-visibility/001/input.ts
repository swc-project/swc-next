export {};
const value = "outer";
function f(): typeof value {
  var value = 1;
  return "outer";
}
