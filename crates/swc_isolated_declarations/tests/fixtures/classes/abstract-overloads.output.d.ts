export declare abstract class Value {
  protected body?(): string | undefined | Promise<string | undefined>;
  protected footer(): string | undefined;
}
declare function overloadsNonExportDecl(args: string): void;
declare function overloadsNonExportDecl(args: number): void;
export { overloadsNonExportDecl };
export default function defaultExport(args: string): void;
export default function defaultExport(args: number): void;
declare class Base {
  method(): void;
  get value(): number;
}
export declare class Derived extends Base {
  method(): void;
  get value(): number;
}
export declare class Overloaded extends Base {
  method(): void;
}
export declare class AmbientDerived extends Base {
  method(): void;
  get value(): number;
}
export declare class AsyncMethods {
  method(value: string): Promise<void>;
  static staticMethod(value: number): Promise<number>;
}
