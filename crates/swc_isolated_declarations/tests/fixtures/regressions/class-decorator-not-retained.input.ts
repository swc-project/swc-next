declare function make(): <T extends new (...args: any[]) => any>(value: T, context: ClassDecoratorContext<T>) => T; const dec = make(); @dec export class C {}
