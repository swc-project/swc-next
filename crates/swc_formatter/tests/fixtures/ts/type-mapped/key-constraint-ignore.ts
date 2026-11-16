type A<T> = { [K /* prettier-ignore */ in /* prettier-ignore */ ( keyof T )]: T[K] };
