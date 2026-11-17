const value = "outer";
function f(x = 0): typeof value {
  enum value {
    A,
  }
  return "outer";
}
