type A<T> = { [ /* prettier-ignore */ K  in keyof T]: T[K] };
