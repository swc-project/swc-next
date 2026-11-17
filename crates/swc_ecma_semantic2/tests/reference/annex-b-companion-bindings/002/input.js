if (condition)
  function f() {
    return f;
  }
const g = f;
f = 1;
g();
