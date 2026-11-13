function f(parameter) {
  var unused = 1;
  eval(code);
  function inner(value) { var local = read(); return [local, value, undefined]; }
  return inner;
}
function g() { var unused = 1; var value = read(); return [value, value, undefined]; }
