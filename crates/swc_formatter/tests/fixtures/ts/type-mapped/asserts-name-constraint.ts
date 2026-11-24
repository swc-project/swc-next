type Keys<T> = { [K in (() => keyof (asserts)) as T]: K };
