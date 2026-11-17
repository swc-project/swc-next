export {};
const value = "outer";
interface I {
  [value: string]: typeof value;
}
