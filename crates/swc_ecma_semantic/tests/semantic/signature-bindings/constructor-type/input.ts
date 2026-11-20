export {};
const value = "outer";
type I = new (value: number) => { value: typeof value };
