export {};
const value = "outer";
type F = <T extends typeof value>(value: number) => T;
