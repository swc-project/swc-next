const value = 1;
function f(x = 0): typeof value {
  function value() {}
  return value;
}
