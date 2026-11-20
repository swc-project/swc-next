export {};
const key = "method";
interface I {
  [key](key: number): typeof key;
}
