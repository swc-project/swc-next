function outer(f) {
  if (condition)
    function f() {
      return f;
    }
  return f;
}
