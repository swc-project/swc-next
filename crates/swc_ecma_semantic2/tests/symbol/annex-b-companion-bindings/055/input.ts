namespace N {
  export function f(value) {}
  if (condition)
    function f() {
      return f;
    }
  f;
}
