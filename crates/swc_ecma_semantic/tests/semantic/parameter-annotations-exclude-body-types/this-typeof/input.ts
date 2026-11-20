const value = 1;
function f(this: typeof value) {
  var value = 2;
}
