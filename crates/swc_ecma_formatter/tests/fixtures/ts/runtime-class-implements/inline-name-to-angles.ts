namespace N { export interface I<T> { x: T; } }
class C implements N . I /* prettier-ignore */ <  number  > { x = 7; }
