declare function fetch<T extends string = string>(this: object, x: T, optional?: number, ...rest: readonly T[]): Promise<T>;
