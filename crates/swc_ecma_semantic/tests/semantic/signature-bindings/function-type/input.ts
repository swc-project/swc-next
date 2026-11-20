export {};
const value = "outer";
type F = (value: number) => typeof value;
