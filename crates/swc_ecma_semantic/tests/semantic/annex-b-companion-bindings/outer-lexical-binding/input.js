let f;
function outer() {
  if (condition)
    function f() {
      return f;
    }
  return f;
}
