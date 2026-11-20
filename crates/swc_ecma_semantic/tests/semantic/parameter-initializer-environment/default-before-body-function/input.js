const value = 1;
function f(x = value) {
  function value() {}
  return value;
}
