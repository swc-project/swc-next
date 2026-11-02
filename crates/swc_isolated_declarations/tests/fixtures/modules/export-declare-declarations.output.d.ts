import { Reference } from "mod";
declare class Class {
}
declare function Function(): Reference;
declare const variable: number;
declare enum Enum {
  A,
  B,
  C,
}
// Should not be included in the output, because they are not exported
export { Class, Function, variable, Enum };
