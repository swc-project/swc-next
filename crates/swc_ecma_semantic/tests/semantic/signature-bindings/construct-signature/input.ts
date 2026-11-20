export {};
const value = "outer";
interface I {
  new (value: number): { value: typeof value };
}
