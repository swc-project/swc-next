export declare const foo: {
  x: string;
  y: string;
  z: string;
  // readonly
  readonly w: string;
};
export declare class Foo {
  get x(): string;
  set x(value: string);
  set y(value: string);
  get y(): string;
  set z(value: string);
  get z(): string;
  // readonly
  get w(): string;
}
