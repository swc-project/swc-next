type Result<T> = ((T | asserts)) extends () => new () => (infer U extends string) ? U : never;
