const value = 1;
function f(...args: (typeof value)[]) {
  const value = 2;
}
