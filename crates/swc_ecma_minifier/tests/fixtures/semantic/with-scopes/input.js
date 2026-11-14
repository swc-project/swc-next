function f(x) {
  with (source) {
    use(x, undefined, function (value) { var local = read(); return [local, value]; });
  }
}
function g() { var unused = 1; var value = read(); return [value, value, undefined]; }
