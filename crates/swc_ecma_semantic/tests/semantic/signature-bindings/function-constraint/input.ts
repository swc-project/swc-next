export {};
const value = "outer";
function f<T extends typeof value>(value: number): T {
  throw 0;
}
