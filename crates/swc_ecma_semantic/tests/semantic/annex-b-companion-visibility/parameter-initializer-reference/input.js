function outer(value = () => f) {
  if (condition)
    function f() {
      return f;
    }
  return f;
}
