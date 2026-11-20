const value = "outer";
function f(): typeof value {
  class value {}
  return "outer";
}
