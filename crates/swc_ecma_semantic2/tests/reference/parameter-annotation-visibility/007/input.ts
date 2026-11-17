export {};
type T = { x: string };
function f({ x }: T) {
  type T = number;
}
