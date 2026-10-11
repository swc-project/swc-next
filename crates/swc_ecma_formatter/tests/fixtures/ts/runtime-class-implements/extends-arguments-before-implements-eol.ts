namespace N { export interface I<T> { x: T; } export class Base<T> { x = 7; } }
class C extends N . Base<  number  > // prettier-ignore
implements N.I<number> { y = 1; }
