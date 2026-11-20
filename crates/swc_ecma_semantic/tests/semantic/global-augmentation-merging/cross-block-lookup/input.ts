export {};
declare global {
  const x: number;
}
declare global {
  const y: typeof x;
}
const z: typeof x = y;
