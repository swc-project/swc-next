export abstract class Value {
    protected body?(): string | undefined | Promise<string | undefined>;
    protected footer(): string | undefined {
        return "";
    }
}
function overloadsNonExportDecl(args: string): void;
function overloadsNonExportDecl(args: number): void;
function overloadsNonExportDecl(args: any): void {}
export { overloadsNonExportDecl };
export default function defaultExport(args: string): void;
export default function defaultExport(args: number): void;
export default function defaultExport(args: any): void {}

class Base {
  method(): void {}
  get value(): number { return 0; }
}

export class Derived extends Base {
  override method(): void {}
  override get value(): number { return 1; }
}

export class Overloaded extends Base {
  override method(): void;
  override method(): void {}
}

export declare class AmbientDerived extends Base {
  override method(): void;
  override get value(): number;
}

export class AsyncMethods {
  async method(value: string): Promise<void>;
  async method(value: unknown): Promise<void> {}

  static async staticMethod(value: number): Promise<number>;
  static async staticMethod(value: unknown): Promise<number> { return 0; }
}
