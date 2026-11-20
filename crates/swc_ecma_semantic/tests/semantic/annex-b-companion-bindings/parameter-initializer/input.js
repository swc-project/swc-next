function outer(value = 1) {
  if (condition)
    function f() {
      return f;
    }
  return f;
}
