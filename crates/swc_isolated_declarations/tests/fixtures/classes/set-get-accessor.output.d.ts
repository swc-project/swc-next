// Correct
declare class Cls {
  get a(): number;
  set a(v: number);
  get b(): string;
  set b(v: string);
  private get c();
  private set c(value);
  accessor d: string;
  private accessor e;
  private static accessor f;
  accessor g: string;
  private accessor h;
  accessor i: string;
}
// Incorrect
declare class ClsBad {
  get a();
  set a(v);
}
/// Infer type from the function body of the getter
declare class GlobalSymbol1 {
  set [Symbol.toStringTag](v: string);
  get [Symbol.toStringTag](): string;
}
// Infer type from the parameter type of the setter
declare class GlobalSymbol2 {
  set [Symbol.toStringTag](v: number);
  get [Symbol.toStringTag](): number;
}
// Infer type from the parameter type of the setter
declare class GlobalSymbol3 {
  set [Symbol.toStringTag](v: number);
  get [Symbol.toStringTag](): string;
}
// Cannot infer type from the function body of the getter
declare class GlobalSymbol4 {
  set [Symbol.toStringTag](v);
  get [Symbol.toStringTag]();
}
// Accessor pairs use equivalent JavaScript property keys.
declare class ComputedStringGetter {
  get value(): string;
  set ["value"](v: string);
}
declare class ComputedStringSetter {
  get ["count"](): number;
  set count(v: number);
}
declare class ComputedNumericGetter {
  get [1](): boolean;
  set [`1`](v: boolean);
}
