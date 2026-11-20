function outer(f = 1) {
  if (condition)
    function f() {
      return f;
    }
  return f;
}
