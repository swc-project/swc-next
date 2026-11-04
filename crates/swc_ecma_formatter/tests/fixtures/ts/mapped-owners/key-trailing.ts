type M<T> = { [K in keyof T]? /* a */: T[K] };
