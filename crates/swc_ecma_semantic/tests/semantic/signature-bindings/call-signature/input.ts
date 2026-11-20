export {};
const value = "outer";
interface I {
  (value: number): typeof value;
}
