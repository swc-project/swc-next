namespace N { export interface I { x: number; } export interface J<T> { x: T; } }
class C implements N . I, /* prettier-ignore */ N . J<  number  > { x = 7; }
