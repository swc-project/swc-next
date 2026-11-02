export declare const foo: {
  x: string;
  y: string;
  z: string;
  readonly w: string;
} // inferred type by annotating the setter parameter
// inferred type by annotating the getter return type
// inferred type by inferring the getter body
// readonly
;
export declare class Foo {
  get x(): string;
  set x(value: string);
  set y(value: string);
  get y(): string;
  set z(value: string);
  get z(): string;
  get w(): string;
  // inferred type by annotating the setter parameter
  // inferred type by annotating the getter return type
  // inferred type by inferring the getter body
  // readonly
}
