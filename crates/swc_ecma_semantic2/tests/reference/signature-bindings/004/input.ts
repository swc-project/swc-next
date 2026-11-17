export {};
const value = "outer";
interface I {
  method(value: number): typeof value;
}
