const value = 1;
function f(arg = () => value) {
  function value() {}
}
