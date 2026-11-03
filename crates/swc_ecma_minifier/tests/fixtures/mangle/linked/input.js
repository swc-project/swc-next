function f(param = 1) {
  var param;
  try { use(); } catch (error) { var error; }
  { function block() {} }
  return [param, error, block];
}
